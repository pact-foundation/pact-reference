#!/usr/bin/env groovy

import groovy.io.FileType
import groovy.json.JsonSlurper

def tests = new File('tests')
def specs = new File(tests, 'spec_testcases')
specs.eachFileRecurse(FileType.DIRECTORIES) { dir ->
  def path = dir.toPath()
  def testFile = new File(dir, 'mod.rs')
  def requestResponsePath = path.getNameCount() > 3 ? path.getName(3).toString() : ''
  def specVersion = path.getName(2).toString().toUpperCase()

  testFile.withPrintWriter { pw ->
    pw.println('#[allow(unused_imports)]')
    pw.println('use test_log::test;')
    pw.println('#[allow(unused_imports)]')
    pw.println('use pact_models::PactSpecification;')
    pw.println('#[allow(unused_imports)]')
    pw.println('use serde_json;')
    pw.println('#[allow(unused_imports)]')
    pw.println('use expectest::prelude::*;')
    pw.println('#[allow(unused_imports)]')
    pw.println('#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager::register_core_entries;')
    if (requestResponsePath == 'request' || requestResponsePath == 'response') {
      pw.println('#[allow(unused_imports)]')
      pw.println('use pact_models::interaction::{Interaction, http_interaction_from_json};')
      pw.println('#[allow(unused_imports)]')
      pw.println('use pact_matching::{match_interaction_request, match_interaction_response};')
      pw.println('#[allow(unused_imports)]')
      pw.println('use pact_models::prelude::{Pact, RequestResponsePact};')
    } else if (requestResponsePath == 'message') {
      pw.println('#[allow(unused_imports)]')
      pw.println('use pact_models::interaction::{Interaction, message_interaction_from_json};')
      pw.println('#[allow(unused_imports)]')
      pw.println('use pact_matching::match_interaction;')
      pw.println('#[allow(unused_imports)]')
      pw.println('use pact_models::prelude::{MessagePact, Pact};')
    }

    dir.eachDir {
      pw.println "mod $it.name;"
    }

    dir.eachFileMatch(~/.*\.json/) {
      def json = new JsonSlurper().parse(it)
      def require = ''
      if (it.name.contains('xml')) {
        require = '\n|#[cfg(feature = "xml")]'
      }
      def testBody = """
        |#[test_log::test(tokio::test)]$require
        |async fn ${it.name.replaceAll(' ', '_').replaceAll('-', '_').replaceAll('\\.json', '')}() {
        |    println!("FILE: ${it}");
        |    #[allow(unused_mut)]
        |    let mut pact: serde_json::Value = serde_json::from_str(r#"
      """
      it.text.eachLine { line ->
        testBody += '|      ' + line + '\n'
      }
      testBody += '|    "#).unwrap();' + '\n'
      if (requestResponsePath == 'request') {
        testBody += """
        |    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
        |    let expected = http_interaction_from_json("$it", &interaction_json, &PactSpecification::$specVersion).unwrap();
        |    println!("EXPECTED: {:?}", expected);
        |    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
        |    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
        |    let actual = http_interaction_from_json("$it", &interaction_json, &PactSpecification::$specVersion).unwrap();
        |    println!("ACTUAL: {:?}", actual);
        |    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
        |    let pact_match = pact.get("match").unwrap();
        |
        |    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
        |    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
        |    let result = match_interaction_request(expected, actual, pact, &PactSpecification::$specVersion).await.unwrap().mismatches();
        |
        |    println!("RESULT: {:?}", result);
        |    if pact_match.as_bool().unwrap() {
        |       expect!(result.iter()).to(be_empty());
        |    } else {
        |       expect!(result.iter()).to_not(be_empty());
        |    }
        """
      } else if (requestResponsePath == 'response') {
        testBody += """
        |    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
        |    let expected = http_interaction_from_json("$it", &interaction_json, &PactSpecification::$specVersion).unwrap();
        |    println!("EXPECTED: {:?}", expected);
        |    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
        |    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
        |    let actual = http_interaction_from_json("$it", &interaction_json, &PactSpecification::$specVersion).unwrap();
        |    println!("ACTUAL: {:?}", actual);
        |    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
        |    let pact_match = pact.get("match").unwrap();
        |
        |    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
        |    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
        |    let result = match_interaction_response(expected, actual, pact, &PactSpecification::$specVersion).await.unwrap();
        |
        |    println!("RESULT: {:?}", result);
        |    if pact_match.as_bool().unwrap() {
        |       expect!(result.iter()).to(be_empty());
        |    } else {
        |       expect!(result.iter()).to_not(be_empty());
        |    }
        """
      } else if (requestResponsePath == 'message') {
        testBody += """
        |    let expected_json = pact.get_mut("expected").unwrap();
        |    let interaction_json = expected_json.as_object_mut().unwrap();
        |    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
        |    let expected = message_interaction_from_json("$it", &expected_json, &PactSpecification::$specVersion).unwrap();
        |    println!("EXPECTED: {:?}", expected);
        |    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
        |    let actual_json = pact.get_mut("actual").unwrap();
        |    let interaction_json = actual_json.as_object_mut().unwrap();
        |    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
        |    let actual = message_interaction_from_json("$it", &actual_json, &PactSpecification::$specVersion).unwrap();
        |    println!("ACTUAL: {:?}", actual);
        |    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
        |    let pact_match = pact.get("match").unwrap();
        |
        |    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
        |    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
        |    let result = match_interaction(expected, actual, pact, &PactSpecification::$specVersion).await.unwrap();
        |
        |    println!("RESULT: {:?}", result);
        |    if pact_match.as_bool().unwrap() {
        |       expect!(result.iter()).to(be_empty());
        |    } else {
        |       expect!(result.iter()).to_not(be_empty());
        |    }
        """
      }
      testBody += '|}'
      pw.println testBody.stripMargin('|')
    }
  }
}
