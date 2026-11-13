//! A minimal recording HTTP/1.1 server, used as the provider under test, the provider state callback and
//! a stand-in Pact broker. Each connection serves one request and closes, which is all the verifier needs.

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Default)]
pub struct Request {
  pub method: String,
  /// Path and query, as sent.
  pub path: String,
  pub headers: Vec<(String, String)>,
  pub body: Vec<u8>,
}

impl Request {
  pub fn header(&self, name: &str) -> Option<&str> {
    self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
  }

  pub fn path_only(&self) -> &str {
    self.path.split('?').next().unwrap_or_default()
  }
}

#[derive(Debug, Clone, Default)]
pub struct Response {
  pub status: u16,
  pub headers: Vec<(String, String)>,
  pub body: Vec<u8>,
}

impl Response {
  pub fn json(status: u16, body: &serde_json::Value) -> Response {
    Response {
      status,
      headers: vec![("Content-Type".into(), "application/json".into())],
      body: body.to_string().into_bytes(),
    }
  }
}

pub type Handler = Arc<dyn Fn(&Request, u16) -> Response + Send + Sync>;

#[derive(Debug)]
pub struct Server {
  pub port: u16,
  requests: Arc<Mutex<Vec<Request>>>,
  task: JoinHandle<()>,
}

impl Server {
  /// Binds 127.0.0.1 on a free port. The handler also gets the port, for absolute links in responses.
  pub async fn start(handler: Handler) -> anyhow::Result<Server> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let requests: Arc<Mutex<Vec<Request>>> = Arc::default();
    let recorded = requests.clone();
    let task = tokio::spawn(async move {
      while let Ok((mut stream, _)) = listener.accept().await {
        let handler = handler.clone();
        let recorded = recorded.clone();
        tokio::spawn(async move {
          let Some(req) = read_request(&mut stream).await else { return };
          let res = handler(&req, port);
          recorded.lock().unwrap().push(req);
          let _ = stream.write_all(&encode(&res)).await;
          let _ = stream.shutdown().await;
        });
      }
    });
    Ok(Server { port, requests, task })
  }

  pub fn url(&self) -> String {
    format!("http://127.0.0.1:{}", self.port)
  }

  pub fn requests(&self) -> Vec<Request> {
    self.requests.lock().unwrap().clone()
  }
}

impl Drop for Server {
  fn drop(&mut self) {
    self.task.abort();
  }
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> Option<Request> {
  let mut buf = Vec::new();
  let mut chunk = [0u8; 8192];
  let head_end = loop {
    if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
      break i;
    }
    let n = stream.read(&mut chunk).await.ok()?;
    if n == 0 {
      return None;
    }
    buf.extend_from_slice(&chunk[..n]);
  };
  let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
  let mut lines = head.split("\r\n");
  let mut start = lines.next()?.split(' ');
  let method = start.next()?.to_string();
  let path = start.next()?.to_string();
  let headers: Vec<(String, String)> = lines
    .filter_map(|l| l.split_once(':').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
    .collect();
  let len: usize = headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
    .and_then(|(_, v)| v.parse().ok()).unwrap_or(0);
  let mut body = buf[head_end + 4..].to_vec();
  while body.len() < len {
    let n = stream.read(&mut chunk).await.ok()?;
    if n == 0 {
      break;
    }
    body.extend_from_slice(&chunk[..n]);
  }
  Some(Request { method, path, headers, body })
}

fn encode(res: &Response) -> Vec<u8> {
  let mut out = format!("HTTP/1.1 {} X\r\n", res.status);
  for (k, v) in &res.headers {
    out.push_str(&format!("{k}: {v}\r\n"));
  }
  out.push_str(&format!("Content-Length: {}\r\nConnection: close\r\n\r\n", res.body.len()));
  let mut bytes = out.into_bytes();
  bytes.extend_from_slice(&res.body);
  bytes
}
