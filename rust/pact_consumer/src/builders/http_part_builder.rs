use std::collections::HashMap;

use pact_models::bodies::OptionalBody;
use pact_models::expression_parser::DataType;
use pact_models::generators::{Generator, GeneratorCategory, Generators};
use pact_models::headers::parse_header;
use pact_models::matchingrules::MatchingRules;
use pact_models::path_exp::DocPath;

use crate::prelude::*;

/// Various methods shared between `RequestBuilder` and `ResponseBuilder`.
pub trait HttpPartBuilder {
    /// (Implementation detail.) This function fetches the mutable state that's
    /// needed to update this builder's `headers`. You should not need to use
    /// this under normal circumstances.
    ///
    /// This function has two return values because its job is to split a single
    /// `&mut` into two `&mut` pointing to sub-objects, which has to be done
    /// carefully in Rust.
    #[doc(hidden)]
    fn headers_and_matching_rules_mut(&mut self) -> (&mut HashMap<String, Vec<String>>, &mut MatchingRules);

    /// (Implementation detail.) This function fetches the mutable state that's
    /// needed to update this builder's `generators`. You should not need to use
    /// this under normal circumstances.
    #[doc(hidden)]
    fn generators(&mut self) -> &mut Generators;

    /// (Implementation detail.) This function fetches the mutable state that's
    /// needed to update this builder's `body`. You should not need to use this
    /// under normal circumstances.
    ///
    /// This function has two return values because its job is to split a single
    /// `&mut` into two `&mut` pointing to sub-objects, which has to be done
    /// carefully in Rust.
    #[doc(hidden)]
    fn body_and_matching_rules_mut(&mut self) -> (&mut OptionalBody, &mut MatchingRules);

    /// Specify a header pattern.
    ///
    /// ```
    /// use pact_consumer::prelude::*;
    /// use pact_consumer::*;
    /// use pact_consumer::builders::RequestBuilder;
    /// use regex::Regex;
    ///
    /// RequestBuilder::default()
    ///     .header("X-Simple", "value")
    ///     .header("X-Digits", term!("^[0-9]+$", "123"));
    /// ```
    #[allow(clippy::option_map_unit_fn)]
    fn header<N, V>(&mut self, name: N, value: V) -> &mut Self
    where
        N: Into<String>,
        V: Into<StringPattern>,
    {
      let name = name.into();
      let value = value.into();
      let example = parse_header(name.as_str(), value.to_example().as_str());
      {
        let (headers, rules) = self.headers_and_matching_rules_mut();
        let entry = headers.keys().cloned().find(|k| k.to_lowercase() == name.to_lowercase());
        if let Some(key) = entry {
          headers.get_mut(&key).map(|val| {
            val.extend(example);
          });
        } else {
          headers.insert(name.clone(), example);
        }
        let mut path = DocPath::root();
        path.push_field(name);
        value.extract_matching_rules(path, rules.add_category("header"))
      }
      self
    }

    /// Specify a header pattern and a generator from provider state.
    ///
    /// ```
    /// use pact_consumer::prelude::*;
    /// use pact_consumer::*;
    /// use pact_consumer::builders::RequestBuilder;
    /// use regex::Regex;
    ///
    /// RequestBuilder::default()
    ///     .header_from_provider_state("X-Simple", "providerState", "value")
    ///     .header_from_provider_state("X-Digits", "providerState", term!("^[0-9]+$", "123"));
    /// ```
    #[allow(clippy::option_map_unit_fn)]
    fn header_from_provider_state<N, E, V>(&mut self, name: N, expression: E, value: V) -> &mut Self
      where
        N: Into<String>,
        E: Into<String>,
        V: Into<StringPattern>,
    {
      let expression = expression.into();
      let sub_category = name.into();
      self.header(&sub_category, value);
      let mut sub_category_path = DocPath::root();
      sub_category_path.push_field(sub_category);
      {
        let generators = self.generators();
        generators.add_generator_with_subcategory(
          &GeneratorCategory::HEADER,
          sub_category_path,
          Generator::ProviderStateGenerator(expression, Some(DataType::STRING)),
        )
      }
      self
    }

    /// Attach a generator to the value at the given path in the body, so it is regenerated on
    /// every test run instead of using the example value from the Pact file.
    ///
    /// ```
    /// use pact_consumer::builders::{HttpPartBuilder, RequestBuilder};
    /// use pact_models::generators::Generator;
    ///
    /// RequestBuilder::default()
    ///     .body_generator("$.id", Generator::Uuid(None));
    /// ```
    ///
    /// A plugin-provided generator is attached the same way, as
    /// `Generator::Plugin { name, values }` - see
    /// [`crate::patterns::PluginRule`] for the matching side of it.
    fn body_generator<P: Into<String>>(&mut self, path: P, generator: Generator) -> &mut Self {
      let path = path.into();
      match DocPath::new(path.as_str()) {
        Ok(doc_path) => {
          self.generators().add_generator_with_subcategory(
            &GeneratorCategory::BODY, doc_path, generator);
        }
        Err(err) => panic!("'{}' is not a valid path expression - {}", path, err)
      }
      self
    }

    /// Set the `Content-Type` header.
    fn content_type<CT>(&mut self, content_type: CT) -> &mut Self
    where
        CT: Into<StringPattern>,
    {
        self.header("content-type", content_type)
    }

    /// Set the `Content-Type` header to `text/html`.
    fn html(&mut self) -> &mut Self {
        self.content_type("text/html")
    }

    /// Set the `Content-Type` header to `application/json; charset=utf-8`,
    /// with enough flexibility to cover common variations.
    fn json_utf8(&mut self) -> &mut Self {
        self.content_type(term!(
            "^application/json; charset=(utf|UTF)-8$",
            "application/json; charset=utf-8"
        ))
    }

    /// Specify a body literal. This does not allow using patterns.
    ///
    /// ```
    /// use pact_consumer::prelude::*;
    /// use pact_consumer::builders::RequestBuilder;
    ///
    /// RequestBuilder::default().body("Hello");
    /// ```
    fn body<B: Into<String>>(&mut self, body: B) -> &mut Self {
        let body = body.into();
        {
            let (body_ref, _) = self.body_and_matching_rules_mut();
            *body_ref = OptionalBody::Present(body.into(), None, None);
        }
        self
    }

  /// Specify a body literal with content type. This does not allow using patterns.
  ///
  /// ```
  /// use pact_consumer::prelude::*;
  /// use pact_consumer::builders::RequestBuilder;
  ///
  /// RequestBuilder::default().body2("Hello", "plain/text");
  /// ```
  fn body2<B: Into<String>>(&mut self, body: B, content_type: B) -> &mut Self {
    let body = body.into();
    {
      let (body_ref, _) = self.body_and_matching_rules_mut();
      *body_ref = OptionalBody::Present(body.into(), content_type.into().parse().ok(), None);
    }
    self
  }

    /// Specify the body as `JsonPattern`, possibly including special matching
    /// rules.
    ///
    /// ```
    /// use pact_consumer::prelude::*;
    /// use pact_consumer::*;
    /// use pact_consumer::builders::RequestBuilder;
    ///
    /// RequestBuilder::default().json_body(json_pattern!({
    ///     "message": like!("Hello"),
    /// }));
    /// ```
    fn json_body<B: Into<JsonPattern>>(&mut self, body: B) -> &mut Self {
        let body = body.into();
        {
            let (body_ref, rules) = self.body_and_matching_rules_mut();
            *body_ref = OptionalBody::Present(body.to_example().to_string().into(), Some("application/json".into()), None);
            body.extract_matching_rules(DocPath::root(), rules.add_category("body"));
        }
        self
    }

  /// Specify a text body (text/plain) matching the given pattern.
  ///
  /// ```
  /// use pact_consumer::prelude::*;
  /// use pact_consumer::builders::RequestBuilder;
  /// use pact_consumer::term;
  ///
  /// RequestBuilder::default().body_matching(term!("^(True|False)$", "True"));
  /// ```
  fn body_matching<P: Into<StringPattern>>(&mut self, body: P) -> &mut Self {
    let body = body.into();
    {
      let (body_ref, rules) = self.body_and_matching_rules_mut();
      *body_ref = OptionalBody::Present(body.to_example_bytes().into(), Some("text/plain".into()), None);
      body.extract_matching_rules(DocPath::root(), rules.add_category("body"));
    }
    self
  }

  /// Specify a text body matching the given pattern with a content type.
  ///
  /// ```
  /// use pact_consumer::prelude::*;
  /// use pact_consumer::builders::RequestBuilder;
  /// use pact_consumer::term;
  ///
  /// RequestBuilder::default().body_matching2(term!("^(True|False)$", "True"), "text/ascii");
  /// ```
  fn body_matching2<P: Into<StringPattern>, B: Into<String>>(&mut self, body: P, content_type: B) -> &mut Self {
    let body = body.into();
    {
      let (body_ref, rules) = self.body_and_matching_rules_mut();
      *body_ref = OptionalBody::Present(body.to_example_bytes().into(), content_type.into().parse().ok(), None);
      body.extract_matching_rules(DocPath::root(), rules.add_category("body"));
    }
    self
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::matchingrules::MatchingRule;
  use pact_models::matchingrules_list;
  use regex::Regex;
  use serde_json::json;

  use crate::builders::{HttpPartBuilder, PactBuilder};
  use crate::patterns::{Like, Term};

  #[test_log::test]
  fn header_pattern() {
    let application_regex = Regex::new("application/.*").unwrap();
    let pattern = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.header(
          "Content-Type",
          Term::new(application_regex, "application/json"),
        );
        i
      })
      .build();
    let good = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.header("Content-Type", "application/xml");
        i
      })
      .build();
    let bad = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.header("Content-Type", "text/html");
        i
      })
      .build();
    assert_requests_match!(good, pattern);
    assert_requests_do_not_match!(bad, pattern);
  }

  #[test]
  fn header_generator() {
    let actual = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.header_from_provider_state(
          "Authorization",
          "token",
          "some-token",
        );
        i
      }).build();

    let expected = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.header("Authorization", "from-provider-state");
        i
      })
      .build();

    let good_context = &mut HashMap::new();
    good_context.insert("token", json!("from-provider-state"));
    assert_requests_with_context_match!(actual, expected, good_context);

    let bad_context = &mut HashMap::new();
    bad_context.insert("token", json!("not-from-provider-state"));
    assert_requests_with_context_do_not_match!(actual, expected, bad_context);
  }

  #[test]
  fn body_literal() {
    let pattern = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.body("Hello");
        i
      })
      .build();
    let good = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.body("Hello");
        i
      })
      .build();
    let bad = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.body("Bye");
        i
      })
      .build();
    assert_requests_match!(good, pattern);
    assert_requests_do_not_match!(bad, pattern);
  }

  #[test_log::test]
  fn json_body_pattern() {
    let pattern = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.json_body(json_pattern!({
                "message": Like::new(json_pattern!("Hello")),
            }));
        i
      })
      .build();
    let good = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.json_body(json_pattern!({ "message": "Goodbye" }));
        i
      })
      .build();
    let bad = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.json_body(json_pattern!({ "message": false }));
        i
      })
      .build();
    assert_requests_match!(good, pattern);
    assert_requests_do_not_match!(bad, pattern);
  }

  #[test]
  fn text_body_pattern() {
    let pact = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.body_matching(term!("^(True|False)$", "True"));
        i
      })
      .build()
      .as_v4_pact().unwrap();
    let interaction = pact.interactions.first()
      .unwrap().as_request_response().unwrap();
    expect!(interaction.request.body.value_as_string().unwrap()).to(be_equal_to("True"));
    expect!(interaction.request.matching_rules.rules_for_category("body").unwrap()).to(
      be_equal_to(matchingrules_list! {
        "body"; "$" => [ MatchingRule::Regex("^(True|False)$".to_string()) ]
      })
    );
  }

  #[test]
  fn header_with_different_case_keys() {
    let pattern = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.header("Content-Type", "application/json");
        i.request.header("content-type", "application/xml");
        i
      })
      .build();
    let interactions = pattern.interactions();
    let first_interaction = interactions.first().unwrap().as_request_response().unwrap();
    expect!(first_interaction.request.headers).to(be_some().value(hashmap!{
      "Content-Type".to_string() => vec![ "application/json".to_string(), "application/xml".to_string() ]
    }));
  }

  #[test]
  fn multi_value_header() {
    let pattern = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
        i.request.header("accept", "application/problem+json, application/json, text/plain, */*");
        i
      })
      .build();
    let interactions = pattern.interactions();
    let first_interaction = interactions.first().unwrap().as_request_response().unwrap();
    expect!(first_interaction.request.headers).to(be_some().value(hashmap!{
      "accept".to_string() => vec![
        "application/problem+json".to_string(),
        "application/json".to_string(),
        "text/plain".to_string(),
        "*/*".to_string()
      ]
    }));
  }
}
