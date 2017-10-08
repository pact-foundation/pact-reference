use pact_matching::models::{Pact, OptionalBody};
use serde_json;
use itertools::Itertools;
use std::collections::HashMap;
use hyper::client::*;
use std::error::Error;
use super::provider_client::join_paths;
use hyper::header::{Accept, qitem, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};
use provider_client::extract_body;
use regex::{Regex, Captures};
use hyper::Url;
use hyper::status::StatusCode;

fn is_true(object: &serde_json::Map<String, serde_json::Value>, field: &String) -> bool {
    match object.get(field) {
        Some(json) => match json {
            &serde_json::Value::Bool(b) => b,
            _ => false
        },
        None => false
    }
}

fn as_string(json: &serde_json::Value) -> String {
    match json {
        &serde_json::Value::String(ref s) => s.clone(),
        _ => format!("{}", json)
    }
}

fn content_type(response: &Response) -> String {
    match response.headers.get::<ContentType>() {
        Some(header) => format!("{}", header),
        None => s!("text/plain")
    }
}

fn json_content_type(response: &Response) -> bool {
    match response.headers.get::<ContentType>() {
        Some(header) => {
            let &ContentType(ref mime) = header;
            match mime.clone() {
                Mime(TopLevel::Application, SubLevel::Json, _) => true,
                Mime(TopLevel::Application, SubLevel::Ext(ext), _) => ext == "hal+json",
                _ => false
            }
        },
        None => false
    }
}

fn find_entry(map: &serde_json::Map<String, serde_json::Value>, key: &String) -> Option<(String, serde_json::Value)> {
    match map.keys().find(|k| k.to_lowercase() == key.to_lowercase() ) {
        Some(k) => map.get(k).map(|v| (key.clone(), v.clone()) ),
        None => None
    }
}

#[derive(Debug, Clone)]
pub enum PactBrokerError {
    LinkError(String),
    ContentError(String),
    IoError(String),
    NotFound(String),
    UrlError(String)
}

impl PartialEq<String> for PactBrokerError {
    fn eq(&self, other: &String) -> bool {
        let message = match self {
            &PactBrokerError::LinkError(ref s) => s.clone(),
            &PactBrokerError::ContentError(ref s) => s.clone(),
            &PactBrokerError::IoError(ref s) => s.clone(),
            &PactBrokerError::NotFound(ref s) => s.clone(),
            &PactBrokerError::UrlError(ref s) => s.clone()
        };
        message == *other
    }
}

impl <'a> PartialEq<&'a str> for PactBrokerError {
    fn eq(&self, other: &&str) -> bool {
        let message = match self {
            &PactBrokerError::LinkError(ref s) => s.clone(),
            &PactBrokerError::ContentError(ref s) => s.clone(),
            &PactBrokerError::IoError(ref s) => s.clone(),
            &PactBrokerError::NotFound(ref s) => s.clone(),
            &PactBrokerError::UrlError(ref s) => s.clone()
        };
        message.as_str() == *other
    }
}

#[derive(Debug, Clone)]
pub struct Link {
    name: String,
    href: Option<String>,
    templated: bool
}

impl Link {

    pub fn from_json(link: &String, link_data: &serde_json::Map<String, serde_json::Value>) -> Link {
        Link {
            name: link.clone(),
            href: find_entry(link_data, &s!("href")).map(|(_, href)| as_string(&href)),
            templated: is_true(link_data, &s!("templated"))
        }
    }

}

pub struct HALClient {
    url: String,
    path_info: Option<serde_json::Value>
}

impl HALClient {

    fn default() -> HALClient {
        HALClient{ url: s!(""), path_info: None }
    }

    fn navigate(&mut self, link: &str, template_values: &HashMap<String, String>) -> Result<serde_json::Value, PactBrokerError> {
        if self.path_info.is_none() {
            self.path_info = Some(try!(self.fetch("/")));
        }
        self.path_info = Some(try!(self.fetch_link(link, template_values)));
        Ok(self.path_info.clone().unwrap())
    }

    fn find_link(&self, link: &str) -> Result<Link, PactBrokerError> {
        match self.path_info {
            None => Err(PactBrokerError::LinkError(format!("No previous resource has been fetched from the pact broker. URL: '{}', LINK: '{}'",
                self.url, link))),
            Some(ref json) => match json.get("_links") {
                Some(json) => match json.get(link) {
                    Some(link_data) => link_data.as_object()
                        .map(|link_data| Link::from_json(&s!(link), &link_data))
                        .ok_or(PactBrokerError::LinkError(format!("Link is malformed, expcted an object but got {}. URL: '{}', LINK: '{}'",
                            link_data, self.url, link))),
                    None => Err(PactBrokerError::LinkError(format!("Link '{}' was not found in the response, only the following links where found: {:?}. URL: '{}', LINK: '{}'",
                        link, json.as_object().unwrap_or(&json!({}).as_object().unwrap()).keys().join(", "), self.url, link)))
                },
                None => Err(PactBrokerError::LinkError(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: '{}'",
                    self.url, link)))
            }
        }
    }

    fn fetch_link(&self, link: &str, template_values: &HashMap<String, String>) -> Result<serde_json::Value, PactBrokerError> {
        let link_data = try!(self.find_link(link));
        self.fetch_url(&link_data, template_values)
    }

    fn fetch_url(&self, link: &Link, template_values: &HashMap<String, String>) -> Result<serde_json::Value, PactBrokerError> {
        let link_url = try!(if link.templated {
            debug!("Link URL is templated");
            self.parse_link_url(&link, template_values)
        } else {
            link.href.clone().ok_or(
                PactBrokerError::LinkError(format!("Link is malformed, there is no href. URL: '{}', LINK: '{}'",
                    self.url, link.name)))
        });
        let base = try!(Url::parse(&self.url).map_err(|err| PactBrokerError::UrlError(format!("{}", err.description()))));
        let url = try!(base.join(&link_url).map_err(|err| PactBrokerError::UrlError(format!("{}", err.description()))));
        self.fetch(&url.path())
    }

    fn fetch(&self, path: &str) -> Result<serde_json::Value, PactBrokerError> {
        debug!("Fetching path '{}' from pact broker", path);
        let client = Client::new();
        let res = client.get(&join_paths(&self.url.clone(), s!(path)))
            .header(Accept(vec![
                qitem(Mime(TopLevel::Application, SubLevel::Ext(s!("hal+json")), vec![])),
                qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
            ]))
            .send();
        match res {
            Ok(mut response) => {
                if response.status.is_success() {
                    if json_content_type(&response) {
                        match extract_body(&mut response) {
                            OptionalBody::Present(body) => serde_json::from_str(&body)
                                    .map_err(|err| PactBrokerError::ContentError(format!("Did not get a valid HAL response body from pact broker path '{}' - {}: {}. URL: '{}'",
                                                                                         path, err.description(), err, self.url))),
                            _ => Err(PactBrokerError::ContentError(format!("Did not get a valid HAL response body from pact broker path '{}'. URL: '{}'",
                                                                          path, self.url)))

                        }
                    } else {
                        Err(PactBrokerError::ContentError(format!("Did not get a HAL response from pact broker path '{}', content type is '{}'. URL: '{}'",
                            path, content_type(&response), self.url)))
                    }
                } else {
                    if response.status == StatusCode::NotFound {
                        Err(PactBrokerError::NotFound(format!("Request to pact broker path '{}' failed: {}. URL: '{}'", path,
                            response.status, self.url)))
                    } else {
                        Err(PactBrokerError::IoError(format!("Request to pact broker path '{}' failed: {}. URL: '{}'", path,
                            response.status, self.url)))
                    }
                }
            },
            Err(err) => Err(PactBrokerError::IoError(format!("Failed to access pact broker path '{}' - {:?}. URL: '{}'",
                path, err.description(), self.url)))
        }
    }

    fn parse_link_url(&self, link: &Link, values: &HashMap<String, String>) -> Result<String, PactBrokerError> {
        match link.href {
            Some(ref href) => {
                debug!("templated URL = {}", href);
                let re = Regex::new(r"\{(\w+)\}").unwrap();
                let final_url = re.replace_all(href, |caps: &Captures| {
                    let lookup = caps.at(1).unwrap();
                    debug!("Looking up value for key '{}'", lookup);
                    match values.get(lookup) {
                        Some(val) => val.clone(),
                        None => {
                            warn!("No value was found for key '{}', mapped values are {:?}",
                                lookup, values);
                            format!("{{{}}}", lookup)
                        }
                    }
                });
                debug!("final URL = {}", final_url);
                Ok(final_url)
            },
            None => Err(PactBrokerError::LinkError(format!("Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '{}', LINK: '{}'",
                self.url, link.name)))
        }
    }

    fn iter_links(&self, link: String) -> Result<Vec<Link>, PactBrokerError> {
        match self.path_info {
            None => Err(PactBrokerError::LinkError(format!("No previous resource has been fetched from the pact broker. URL: '{}', LINK: '{}'",
                self.url, link))),
            Some(ref json) => match json.get("_links") {
                Some(json) => match json.get(&link) {
                    Some(link_data) => link_data.as_array()
                        .map(|link_data| link_data.iter().map(|link_json| match link_json {
                            &serde_json::Value::Object(ref data) => Link::from_json(&link, data),
                            &serde_json::Value::String(ref s) => Link { name: link.clone(), href: Some(s.clone()), templated: false },
                            _ => Link { name: link.clone(), href: Some(link_json.to_string()), templated: false }
                        }).collect())
                        .ok_or(PactBrokerError::LinkError(format!("Link is malformed, expcted an object but got {}. URL: '{}', LINK: '{}'",
                            link_data, self.url, link))),
                    None => Err(PactBrokerError::LinkError(format!("Link '{}' was not found in the response, only the following links where found: {:?}. URL: '{}', LINK: '{}'",
                        link, json.as_object().unwrap_or(&json!({}).as_object().unwrap()).keys().join(", "), self.url, link)))
                },
                None => Err(PactBrokerError::LinkError(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: '{}'",
                    self.url, link)))
            }
        }
    }
}

pub fn fetch_pacts_from_broker(broker_url: &String, provider_name: &String) -> Result<Vec<Result<Pact, PactBrokerError>>, PactBrokerError> {
    let mut client = HALClient{ url: broker_url.clone(), .. HALClient::default() };
    let template_values = hashmap!{ s!("provider") => provider_name.clone() };
    match client.navigate("pb:latest-provider-pacts", &template_values) {
        Ok(_) => {
            let pact_links = try!(client.iter_links(s!("pacts")));
            debug!("Pact links = {:?}", pact_links);
            let pacts = pact_links.iter().map(|link| match link.clone().href {
                Some(_) => client.fetch_url(&link, &template_values).map(|pact_json| Pact::from_json(&link.href.clone().unwrap(), &pact_json)),
                None => Err(PactBrokerError::LinkError(format!("Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '{}', LINK: '{:?}'",
                    client.url, link)))
            }).collect();
            debug!("pacts = {:?}", pacts);
            Ok(pacts)
        },
        Err(err) => match err {
            PactBrokerError::NotFound(_) => Err(
                PactBrokerError::NotFound(
                    format!("No pacts for provider '{}' where found in the pact broker. URL: '{}'",
                        provider_name, broker_url))),
            _ => Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use expectest::prelude::*;
    use super::*;
    use super::{content_type, json_content_type};
    use pact_consumer::prelude::*;
    use env_logger::*;
    use pact_matching::models::{Pact, Consumer, Provider, Interaction};
    use hyper::Url;
    use hyper::client::response::Response;
    use std::io::{self, Write, Read};
    use hyper::http::{
        RawStatus,
        HttpMessage,
        RequestHead,
        ResponseHead,
    };
    use hyper::error::Error;
    use hyper::version::HttpVersion;
    use std::time::Duration;
    use hyper::header::{Headers, ContentType};
    use std::borrow::Cow;
    use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

    #[test]
    fn fetch_returns_an_error_if_there_is_no_pact_broker() {
        let client = HALClient{ url: s!("http://idont.exist:6666"), .. HALClient::default() };
        expect!(client.fetch(&s!("/"))).to(be_err());
    }

    #[test]
    fn fetch_returns_an_error_if_it_does_not_get_a_success_response() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
            .interaction("a request to a non-existant path", |i| {
                i.given("the pact broker has a valid pact");
                i.request.path("/hello");
                i.response.status(404);
            })
            .start_mock_server();

        let client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = client.fetch(&s!("/hello"));
        expect!(result).to(be_err().value(format!("Request to pact broker path \'/hello\' failed: 404 Not Found. URL: '{}'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_returns_an_error_if_it_does_not_get_a_hal_response() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-json resource", |i| {
                i.request.path("/nonjson");
                i.response
                    .header("Content-Type", "text/html")
                    .body("<html></html>");
            })
            .start_mock_server();

        let client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = client.fetch(&s!("/nonjson"));
        expect!(result).to(be_err().value(format!("Did not get a HAL response from pact broker path \'/nonjson\', content type is 'text/html'. URL: '{}'",
            pact_broker.url())));
    }

    #[derive(Debug, Clone)]
    struct MockHttpMessage {
        pub body: Option<String>,
        pub headers: Headers,
        pub status: RawStatus
    }

    impl HttpMessage for MockHttpMessage {

        fn set_outgoing(&mut self, _head: RequestHead) -> Result<RequestHead, Error> {
            Err(Error::Io(io::Error::new(io::ErrorKind::Other, "Not supported with MockHttpMessage")))
        }

        fn get_incoming(&mut self) -> Result<ResponseHead, Error> {
            Ok(ResponseHead {
                headers: self.headers.clone(),
                raw_status: self.status.clone(),
                version: HttpVersion::Http11,
            })
        }

        fn has_body(&self) -> bool {
            self.body.is_some()
        }

        fn set_read_timeout(&self, _dur: Option<Duration>) -> io::Result<()> {
            Ok(())
        }

        fn set_write_timeout(&self, _dur: Option<Duration>) -> io::Result<()> {
            Ok(())
        }

        fn close_connection(&mut self) -> Result<(), Error> {
            Ok(())
        }
    }

    impl Write for MockHttpMessage {

        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Other, "Not supported with MockHttpMessage"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Other, "Not supported with MockHttpMessage"))
        }

    }

    impl Read for MockHttpMessage {

        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Other, "Not supported with MockHttpMessage"))
        }

    }

    #[test]
    fn content_type_test() {
        let mut message = MockHttpMessage {
            body: None,
            status: RawStatus(200, Cow::Owned(s!("OK"))),
            headers: Headers::new()
        };
        let url = Url::parse("http://localhost").unwrap();

        let response = Response::with_message(url.clone(), Box::new(message.clone())).unwrap();
        expect!(content_type(&response)).to(be_equal_to(s!("text/plain")));

        message.headers.set::<ContentType>(
            ContentType(Mime(TopLevel::Application, SubLevel::Ext(s!("hal+json")),
                vec![(Attr::Charset, Value::Utf8)])));
        let response = Response::with_message(url.clone(), Box::new(message.clone())).unwrap();
        expect!(content_type(&response)).to(be_equal_to(s!("application/hal+json; charset=utf-8")));
    }

    #[test]
    fn json_content_type_test() {
        let mut message = MockHttpMessage {
            body: None,
            status: RawStatus(200, Cow::Owned(s!("OK"))),
            headers: Headers::new()
        };
        let url = Url::parse("http://localhost").unwrap();

        let response = Response::with_message(url.clone(), Box::new(message.clone())).unwrap();
        expect!(json_content_type(&response)).to(be_false());

        message.headers.set::<ContentType>(
            ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
        let response = Response::with_message(url.clone(), Box::new(message.clone())).unwrap();
        expect!(json_content_type(&response)).to(be_true());

        message.headers.set::<ContentType>(
            ContentType(Mime(TopLevel::Application, SubLevel::Ext(s!("hal+json")),
                vec![(Attr::Charset, Value::Utf8)])));
        let response = Response::with_message(url.clone(), Box::new(message.clone())).unwrap();
        expect!(json_content_type(&response)).to(be_true());
    }

    #[test]
    fn fetch_returns_an_error_if_it_does_not_get_a_valid_hal_response() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-hal resource", |i| {
                i.request.path("/nonhal");
                i.response.header("Content-Type", "application/hal+json");
            })
            .interaction("a request to a non-hal resource 2", |i| {
                i.request.path("/nonhal2");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("<html>This is not JSON</html>");
            })
            .start_mock_server();

        let client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = client.fetch(&s!("/nonhal"));
        expect!(result).to(be_err().value(format!("Did not get a valid HAL response body from pact broker path \'/nonhal\'. URL: '{}'",
            pact_broker.url())));
        let result = client.fetch(&s!("/nonhal2"));
        expect!(result).to(be_err().value(format!("Did not get a valid HAL response body from pact broker path \'/nonhal2\' - JSON error: expected value at line 1 column 1. URL: '{}'",
            pact_broker.url())));
    }

    #[test]
    fn parse_link_url_returns_error_if_there_is_no_href() {
        let client = HALClient::default();
        let link = Link { name: s!("link"), href: None, templated: false };
        expect!(client.parse_link_url(&link, &hashmap!{})).to(be_err().value(
            "Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '', LINK: 'link'"));
    }

    #[test]
    fn parse_link_url_replaces_all_tokens_in_href() {
        let client = HALClient::default();
        let values = hashmap!{ s!("valA") => s!("A"), s!("valB") => s!("B") };

        let link = Link { name: s!("link"), href: Some(s!("http://localhost")), templated: false };
        expect!(client.parse_link_url(&link, &values)).to(be_ok().value("http://localhost"));

        let link = Link { name: s!("link"), href: Some(s!("http://{valA}/{valB}")), templated: false };
        expect!(client.parse_link_url(&link, &values)).to(be_ok().value("http://A/B"));

        let link = Link { name: s!("link"), href: Some(s!("http://{valA}/{valC}")), templated: false };
        expect!(client.parse_link_url(&link, &values)).to(be_ok().value("http://A/{valC}"));
    }

    #[test]
    fn fetch_link_returns_an_error_if_a_previous_resource_has_not_been_fetched() {
        let client = HALClient{ url: s!("http://localhost"), .. HALClient::default() };
        let result = client.fetch_link(&s!("anything_will_do"), &hashmap!{});
        expect!(result).to(be_err().value(s!("No previous resource has been fetched from the pact broker. URL: 'http://localhost', LINK: 'anything_will_do'")));
    }

    #[test]
    fn fetch_link_returns_an_error_if_the_previous_resource_was_not_hal() {
        init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-hal json resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{}");
            })
            .start_mock_server();

        let mut client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = client.fetch(&s!("/"));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.fetch_link(&s!("hal2"), &hashmap!{});
        expect!(result).to(be_err().value(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: 'hal2'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_link_returns_an_error_if_the_previous_resource_links_are_not_correctly_formed() {
        init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource with invalid links", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":[{\"next\":{\"href\":\"abc\"}},{\"prev\":{\"href\":\"def\"}}]}");
            })
            .start_mock_server();

        let mut client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = client.fetch(&s!("/"));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.fetch_link(&s!("any"), &hashmap!{});
        expect!(result).to(be_err().value(format!("Link 'any' was not found in the response, only the following links where found: \"\". URL: '{}', LINK: 'any'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_link_returns_an_error_if_the_previous_resource_does_not_have_the_link() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"next\":{\"href\":\"/abc\"},\"prev\":{\"href\":\"/def\"}}}");
            })
            .start_mock_server();

        let mut client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = client.fetch(&s!("/"));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.fetch_link(&s!("any"), &hashmap!{});
        expect!(result).to(be_err().value(format!("Link 'any' was not found in the response, only the following links where found: \"next, prev\". URL: '{}', LINK: 'any'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_link_returns_the_resource_for_the_link() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"next\":{\"href\":\"/abc\"},\"prev\":{\"href\":\"/def\"}}}");
            })
            .interaction("a request to next", |i| {
                i.request.path("/abc");
                i.response
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!("Yay! You found your way here"));
            })
            .start_mock_server();

        let mut client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = client.fetch(&s!("/"));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.fetch_link(&s!("next"), &hashmap!{});
        expect!(result).to(be_ok().value(serde_json::Value::String(s!("Yay! You found your way here"))));
    }

    #[test]
    fn fetch_link_returns_handles_absolute_resource_links() {
        init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource with absolute paths", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"next\":{\"href\":\"http://localhost/abc\"},\"prev\":{\"href\":\"http://localhost/def\"}}}");
            })
            .interaction("a request to next", |i| {
                i.request.path("/abc");
                i.response
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!("Yay! You found your way here"));
            })
            .start_mock_server();

        let mut client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = client.fetch(&s!("/"));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.fetch_link(&s!("next"), &hashmap!{});
        expect!(result).to(be_ok().value(serde_json::Value::String(s!("Yay! You found your way here"))));
    }

    #[test]
    fn fetch_link_returns_the_resource_for_the_templated_link() {
        init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a templated hal resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"document\":{\"href\":\"/doc/{id}\",\"templated\":true}}}");

            })
            .interaction("a request for a document", |i| {
                i.request.path("/doc/abc");
                i.response
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!("Yay! You found your way here"));
            })
            .start_mock_server();

        let mut client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = client.fetch(&s!("/"));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.fetch_link(&s!("document"), &hashmap!{ s!("id") => s!("abc") });
        expect!(result).to(be_ok().value(serde_json::Value::String(s!("Yay! You found your way here"))));
    }

    #[test]
    fn fetch_pacts_from_broker_returns_empty_list_if_there_are_no_pacts() {
        init().unwrap_or(());

        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
            .interaction("a request to the pact broker root", |i| {
                i.request
                    .path("/")
                    .header("Accept", "application/hal+json, application/json");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .json_body(json_pattern!({
                        "_links": {
                            "pb:latest-provider-pacts": {
                                "href": "http://localhost/pacts/provider/{provider}/latest",
                                "templated": true,
                            }
                        }
                    }));
            })
            .interaction("a request for a providers pacts", |i| {
                i.given("There are no pacts in the pact broker");
                i.request
                    .path("/pacts/provider/sad_provider/latest")
                    .header("Accept", "application/hal+json, application/json");
                i.response.status(404);
            })
            .start_mock_server();

        let result = fetch_pacts_from_broker(&pact_broker.url().to_string(), &s!("sad_provider"));
        expect!(result).to(be_err().value(format!("No pacts for provider 'sad_provider' where found in the pact broker. URL: '{}'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_pacts_from_broker_returns_a_list_of_pacts() {
        init().unwrap_or(());

        let pact = Pact { consumer: Consumer { name: s!("Consumer") },
            provider: Provider { name: s!("happy_provider") },
            .. Pact::default() }
            .to_json().to_string();
        let pact2 = Pact { consumer: Consumer { name: s!("Consumer2") },
            provider: Provider { name: s!("happy_provider") },
            interactions: vec![ Interaction { description: s!("a request friends"), .. Interaction::default() } ],
            .. Pact::default() }
            .to_json().to_string();
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
            .interaction("a request to the pact broker root", |i| {
                i.request
                    .path("/")
                    .header("Accept", "application/hal+json, application/json");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .json_body(json_pattern!({
                        "_links": {
                            "pb:latest-provider-pacts": {
                                "href": "http://localhost/pacts/provider/{provider}/latest",
                                "templated": true,
                            }
                        }
                    }));
            })
            .interaction("a request for a providers pacts", |i| {
                i.given("There are two pacts in the pact broker");
                i.request
                    .path("/pacts/provider/happy_provider/latest")
                    .header("Accept", "application/hal+json, application/json");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .json_body(json_pattern!({
                        "_links":{
                            "pacts":[
                                {"href":"http://localhost/pacts/provider/happy_provider/consumer/Consumer/version/1.0.0"},
                                {"href":"http://localhost/pacts/provider/happy_provider/consumer/Consumer2/version/1.0.0"}
                            ]
                        }
                    }));
            })
            .interaction("a request for the first provider pact", |i| {
                i.given("There are two pacts in the pact broker");
                i.request
                    .path("/pacts/provider/happy_provider/consumer/Consumer/version/1.0.0")
                    .header("Accept", "application/hal+json, application/json");
                i.response
                    .header("Content-Type", "application/json")
                    .body(pact.clone());
            })
            .interaction("a request for the second provider pact", |i| {
                i.given("There are two pacts in the pact broker");
                i.request
                    .path("/pacts/provider/happy_provider/consumer/Consumer2/version/1.0.0")
                    .header("Accept", "application/hal+json, application/json");
                i.response
                    .header("Content-Type", "application/json")
                    .body(pact2.clone());
            })
            .start_mock_server();

        let result = fetch_pacts_from_broker(&pact_broker.url().to_string(), &s!("happy_provider"));
        expect!(result.clone()).to(be_ok());
        let pacts = result.unwrap();
        expect!(pacts.len()).to(be_equal_to(2));
        for pact in pacts {
            expect!(pact).to(be_ok());
        }
    }
}
