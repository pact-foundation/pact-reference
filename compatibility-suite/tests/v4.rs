use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use cucumber::World;
use itertools::Itertools;
use rocket::http::{ContentType, Header};
use rocket::Responder;
use rocket::serde::json::Json;
use serde::Deserialize;
use serde_json::json;
use tracing::debug;
use tracing_subscriber::EnvFilter;

use crate::v4_steps::V4World;

mod shared_steps;
mod v4_steps;

#[derive(Deserialize, Default, Debug)]
struct MessageDetails {
  description: String
}

#[derive(Responder)]
struct MessageResponder<'a> {
  payload: Option<Vec<u8>>,
  content_type: ContentType,
  metadata: Header<'a>
}

#[rocket::post("/", data = "<request>")]
async fn messages(request: Json<MessageDetails>) -> Option<MessageResponder<'static>> {
  let details = request.into_inner();
  debug!("Got request = {:?}", details);
  let guard = v4_steps::message_provider::MESSAGES.lock().unwrap();
  guard.get(details.description.as_str())
    .map(|message| {
      let metadata = json!(message.contents.metadata).to_string();
      MessageResponder {
        payload: message.contents.contents.value().map(|data| data.to_vec()),
        content_type: message.message_content_type()
          .map(|ct| ContentType::parse_flexible(ct.to_string().as_str()))
          .flatten()
          .unwrap_or(ContentType::Plain),
        metadata: Header::new("pact-message-metadata", BASE64.encode(metadata))
      }
    })
}

#[tokio::main]
async fn main() {
  let format = tracing_subscriber::fmt::format().pretty();
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .event_format(format)
    .init();

  let server = rocket::build()
    .mount("/", rocket::routes![messages])
    .ignite()
    .await.expect("Could not start the Rocket server");
  let shutdown = server.shutdown();
  let port = server.config().port;
  tokio::spawn(server.launch());

  V4World::cucumber()
    .fail_on_skipped()
    .before(move |_, _, scenario, world| Box::pin(async move {
      world.scenario_id = scenario.name.clone();
      world.message_proxy_port = port;
    }))
    .after(|_feature, _, _scenario, _status, world| Box::pin(async move {
      if let Some(world) = world {
        let _ = world.provider_server.shutdown();

        let mut guard = v4_steps::message_provider::MESSAGES.lock().unwrap();
        let keys = guard.keys().cloned().collect_vec();
        for key in keys {
          if key.starts_with(world.scenario_id.as_str()) {
            guard.remove(key.as_str());
          }
        }
      }
    }))
    .run_and_exit("pact-compatibility-suite/features/V4")
    .await;

  shutdown.notify();
}
