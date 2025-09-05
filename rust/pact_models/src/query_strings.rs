use std::collections::HashMap;
use std::str::from_utf8;

use hex::FromHex;
use itertools::Itertools;
use serde_json::Value;
use tracing::{error, trace, warn};

use crate::PactSpecification;

/// Decodes a query string using a percent-encoding scheme
pub fn decode_query(query: &str) -> Result<String, String> {
  let mut chars = query.chars();
  let mut ch = chars.next();
  let mut buffer = vec![];

  while ch.is_some() {
    let c = ch.unwrap();
    trace!("ch = '{:?}'", ch);
    if c == '%' {
      let c1 = chars.next();
      let c2 = chars.next();
      match (c1, c2) {
        (Some(v1), Some(v2)) => {
          let mut s = String::new();
          s.push(v1);
          s.push(v2);
          let decoded: Result<Vec<u8>, _> = FromHex::from_hex(s.into_bytes());
          match decoded {
            Ok(n) => {
              trace!("decoded = '{:?}'", n);
              buffer.extend_from_slice(&n);
            },
            Err(err) => {
              error!("Failed to decode '%{}{}' to as HEX - {}", v1, v2, err);
              buffer.push('%' as u8);
              buffer.push(v1 as u8);
              buffer.push(v2 as u8);
            }
          }
        },
        (Some(v1), None) => {
          buffer.push('%' as u8);
          buffer.push(v1 as u8);
        },
        _ => buffer.push('%' as u8)
      }
    } else if c == '+' {
      buffer.push(' ' as u8);
    } else {
      buffer.push(c as u8);
    }

    ch = chars.next();
  }

  match from_utf8(&buffer) {
    Ok(s) => Ok(s.to_owned()),
    Err(err) => {
      error!("Failed to decode '{}' to UTF-8 - {}", query, err);
      Err(format!("Failed to decode '{}' to UTF-8 - {}", query, err))
    }
  }
}

/// Encodes the query string using a percent-encoding scheme
pub fn encode_query(query: &str) -> String {
  query.chars().map(|ch| {
    match ch {
      ' ' => "+".to_string(),
      '-' => ch.to_string(),
      'a'..='z' => ch.to_string(),
      'A'..='Z' => ch.to_string(),
      '0'..='9' => ch.to_string(),
      _ => ch.escape_unicode()
        .filter(|u| u.is_digit(16))
        .batching(|it| {
          match it.next() {
            None => None,
            Some(x) => Some((x, it.next().unwrap()))
          }
        })
        .map(|u| format!("%{}{}", u.0, u.1))
        .collect()
    }
  }).collect()
}

/// Parses a query string into an optional map. The query parameter name will be mapped to
/// a list of values. Where the query parameter is repeated, the order of the values will be
/// preserved.
pub fn parse_query_string(query: &str) -> Option<HashMap<String, Vec<Option<String>>>> {
  if !query.is_empty() {
    Some(query.split('&').map(|kv| {
      trace!("kv = '{}'", kv);
      if kv.is_empty() {
        vec![]
      } else {
        kv.splitn(2, '=').collect::<Vec<&str>>()
      }
    }).fold(HashMap::new(), |mut map, name_value| {
      trace!("name_value = '{:?}'", name_value);
      if !name_value.is_empty() {
        let name = decode_query(name_value[0])
          .unwrap_or_else(|_| name_value[0].to_owned());
        let value = if name_value.len() > 1 {
          Some(decode_query(name_value[1]).unwrap_or_else(|_| name_value[1].to_owned()))
        } else {
          None
        };
        trace!("decoded: '{}' => {:?}", name, value);
        map.entry(name).or_insert_with(|| vec![]).push(value);
      }
      map
    }))
  } else {
    None
  }
}

/// Converts a query string map into a query string
pub fn build_query_string(query: HashMap<String, Vec<Option<String>>>) -> String {
  // Collect all (key, index, value) triples to preserve the order of values
  let mut entries = vec![];
  for (key, values) in &query {
    for (idx, value) in values.iter().enumerate() {
      entries.push((idx, key, value));
    }
  }
  // Sort by index to preserve the order of values across keys
  entries.sort_by_key(|(idx, _, _)| *idx);

  // Sort by index, then by key lexicographically for each index
  entries.sort_by(|(idx_a, key_a, _), (idx_b, key_b, _)| {
    idx_a.cmp(idx_b).then_with(|| key_a.cmp(key_b))
  });

  entries.into_iter()
    .filter(|(_, k, _)| !k.is_empty())
    .map(|(_, k, v)| match v {
      None => k.clone(),
      Some(s) => format!("{}={}", k, encode_query(s))
    })
    .collect::<Vec<_>>()
    .join("&")
}

/// Parses a V2 query string from a JSON struct
pub fn query_from_json(query_json: &Value, spec_version: &PactSpecification) -> Option<HashMap<String, Vec<Option<String>>>> {
  match query_json {
    Value::String(s) => parse_query_string(s),
    _ => {
      warn!("Only string versions of request query strings are supported with specification version {}, ignoring.",
        spec_version.to_string());
      None
    }
  }
}

/// Parses a V3 query string from a JSON struct
pub fn v3_query_from_json(
  query_json: &Value,
  spec_version: &PactSpecification
) -> Option<HashMap<String, Vec<Option<String>>>> {
  match query_json {
    Value::String(s) => parse_query_string(s),
    Value::Object(map) => Some(map.iter().map(|(k, v)| {
      (k.clone(), match v {
        Value::String(s) => vec![Some(s.clone())],
        Value::Array(array) => array.iter().map(|item| match item {
          Value::String(s) => Some(s.clone()),
          Value::Null => None,
          _ => Some(v.to_string())
        }).collect(),
        _ => {
          warn!("Query parameter value '{}' is not valid, ignoring", v);
          vec![]
        }
      })
    }).collect()),
    _ => {
      warn!("Only string or map versions of request query strings are supported with specification version {}, ignoring.",
                spec_version.to_string());
      None
    }
  }
}

/// Converts a query string structure into a JSON struct
pub fn query_to_json(query: HashMap<String, Vec<Option<String>>>, spec_version: &PactSpecification) -> Value {
  match spec_version {
    PactSpecification::V3 | PactSpecification::V4 => Value::Object(query
      .iter()
      .sorted_by(|(a, _), (b, _)| Ord::cmp(a, b))
      .map(|(k, v)| {
        (k.clone(), Value::Array(v.iter().map(|q| match q {
          None => Value::Null,
          Some(s) => Value::String(s.clone())
        }).collect()))
      })
      .collect()),
    _ => Value::String(build_query_string(query))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use expectest::prelude::*;
  use maplit::hashmap;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  use crate::query_strings::parse_query_string;
use crate::query_strings::build_query_string;

  #[test]
  fn parse_query_string_test() {
    let query = "a=b&c=d".to_string();
    let expected = hashmap!{
    "a".to_string() => vec![Some("b".to_string())],
    "c".to_string() => vec![Some("d".to_string())]
  };
    let result = parse_query_string(&query);
    expect!(result).to(be_some().value(expected));
  }

  #[test]
  fn parse_query_string_handles_empty_string() {
    let query = "".to_string();
    let expected = None;
    let result = parse_query_string(&query);
    assert_eq!(result, expected);
  }

  #[test]
  fn parse_query_string_handles_missing_values() {
    let query = "a=&c=d".to_string();
    let mut expected = HashMap::new();
    expected.insert("a".to_string(), vec![Some("".to_string())]);
    expected.insert("c".to_string(), vec![Some("d".to_string())]);
    let result = parse_query_string(&query);
    assert_eq!(result, Some(expected));
  }

  #[test]
  fn parse_query_string_handles_equals_in_values() {
    let query = "a=b&c=d=e=f".to_string();
    let mut expected = HashMap::new();
    expected.insert("a".to_string(), vec![Some("b".to_string())]);
    expected.insert("c".to_string(), vec![Some("d=e=f".to_string())]);
    let result = parse_query_string(&query);
    assert_eq!(result, Some(expected));
  }

  #[test]
  fn parse_query_string_decodes_values() {
    let query = "a=a%20b%20c".to_string();
    let expected = hashmap! {
    "a".to_string() => vec![Some("a b c".to_string())]
  };
    let result = parse_query_string(&query);
    expect!(result).to(be_some().value(expected));
  }

  #[test]
  fn parse_query_string_decodes_non_ascii_values() {
    let query = "accountNumber=100&anotherValue=%E6%96%87%E4%BB%B6.txt".to_string();
    let expected = hashmap! {
    "accountNumber".to_string() => vec![Some("100".to_string())],
    "anotherValue".to_string() => vec![Some("文件.txt".to_string())]
  };
    let result = parse_query_string(&query);
    expect!(result).to(be_some().value(expected));
  }

  #[test]
  fn parse_query_string_handles_no_values() {
    let query = "a&c=&c&a".to_string();
    let mut expected = HashMap::new();
    expected.insert("a".to_string(), vec![None, None]);
    expected.insert("c".to_string(), vec![Some("".to_string()), None]);
    let result = parse_query_string(&query);
    assert_eq!(result, Some(expected));
  }

  #[rstest]
  #[case(hashmap!{}, "")]
  #[case(hashmap!{ "A".to_string() => vec![] }, "")]
  #[case(hashmap!{ "A".to_string() => vec![ Some("B".to_string()) ] }, "A=B")]
  #[case(hashmap!{ "".to_string() => vec![ Some("B".to_string()) ] }, "")]
  #[case(hashmap!{ "A".to_string() => vec![ Some("B".to_string()), Some("c".to_string()) ] }, "A=B&A=c")]
  #[case(hashmap!{ "A".to_string() => vec![ Some("B".to_string()) ], "b".to_string() => vec![ Some("c".to_string()) ] }, "A=B&b=c")]
  #[case(hashmap!{ "A".to_string() => vec![ Some("".to_string()) ] }, "A=")]
  #[case(hashmap!{ "A".to_string() => vec![ Some("".to_string()), Some("".to_string()) ] }, "A=&A=")]
  #[case(hashmap!{ "A".to_string() => vec![ None ] }, "A")]
  #[case(hashmap!{ "A".to_string() => vec![ None, None ] }, "A&A")]
  fn build_query_string_test(#[case] map: HashMap<String, Vec<Option<String>>>, #[case] expected: &str) {
    let result = super::build_query_string(map);
    assert_eq!(result, expected)
  }

  #[test]
  fn build_query_string_multiple_keys_and_values_ordered() {
    let mut query = HashMap::new();
    query.insert("pacticipant".to_string(), vec![Some("foo".to_string()), Some("bar".to_string())]);
    query.insert("version".to_string(), vec![Some("1.2.3".to_string()), Some("4.5.6".to_string())]);
    // The order must be: pacticipant=foo&version=1.2.3&pacticipant=bar&version=4.5.6
    let result = build_query_string(query);
    assert_eq!(result, "pacticipant=foo&version=1%2e2%2e3&pacticipant=bar&version=4%2e5%2e6");
  }


    // sends URL Querying broker at: http://127.0.0.1:61567/matrix?q[][pacticipant]=Foo&q[][version]=1.2.3&q[][pacticipant]=Bar&q[][latest]=true&q[][tag]=prod&latestby=cvpv
    // pact saves url as latestby=cvpv&q[][latest]=true&q[][pacticipant]=Foo&q[][tag]=prod&q[][version]=1%2e2%2e3&q[][pacticipant]=Bar
    // pact should save url as q[][pacticipant]=Foo&q[][version]=1%2e2%2e3&q[][pacticipant]=Bar&q[][latest]=true&q[][tag]=prod&latestby=cvpv
    // pact-ruby saves q%5B%5D%5Bpacticipant%5D=Foo&q%5B%5D%5Bversion%5D=1.2.3&q%5B%5D%5Bpacticipant%5D=Bar&q%5B%5D%5Blatest%5D=true&q%5B%5D%5Btag%5D=prod&latestby=cvpv 


      // .query_param("q[][pacticipant]", "Foo")
      // .query_param("q[][version]", "1.2.3")
      // .query_param("q[][pacticipant]", "Bar")
      // .query_param("q[][latest]", "true")
      // .query_param("q[][tag]", "prod")
      // .query_param("latestby", "cvpv");


    #[test]
    fn build_query_string_matrix_query_with_multiple_q_params_and_latestby() {
      // This test checks that the query string is built in the expected order for matrix queries
      // with multiple q[][param] and a latestby param.
      let mut query = HashMap::new();
      query.insert("q[][pacticipant]".to_string(), vec![Some("Foo".to_string()), Some("Bar".to_string())]);
      query.insert("q[][version]".to_string(), vec![Some("1.2.3".to_string())]);
      query.insert("q[][latest]".to_string(), vec![Some("true".to_string())]);
      query.insert("q[][tag]".to_string(), vec![Some("prod".to_string())]);
      query.insert("latestby".to_string(), vec![Some("cvpv".to_string())]);
      // The expected order is: q[][pacticipant]=Foo&q[][version]=1%2e2%2e3&q[][pacticipant]=Bar&q[][latest]=true&q[][tag]=prod&latestby=cvpv
      let result = build_query_string(query);
      assert_eq!(
        result,
        "q[][pacticipant]=Foo&q[][version]=1%2e2%2e3&q[][pacticipant]=Bar&q[][latest]=true&q[][tag]=prod&latestby=cvpv"
      );
    }


    // sends URL Querying broker at: http://127.0.0.1:61517/matrix?q[][pacticipant]=Foo&q[][version]=1.2.4&q[][pacticipant]=Bar&q[][latest]=true&latestby=cvpv
    // pact saves URL as latestby=cvpv&q[][latest]=true&q[][pacticipant]=Foo&q[][version]=1%2e2%2e4&q[][pacticipant]=Bar
    // pact should save url as q[][pacticipant]=Foo&q[][version]=1%2e2%2e4&q[][pacticipant]=Bar&q[][latest]=true&latestby=cvpv
    // pact-ruby saves q%5B%5D%5Bpacticipant%5D=Foo&q%5B%5D%5Bversion%5D=1.2.4&q%5B%5D%5Bpacticipant%5D=Bar&q%5B%5D%5Blatest%5D=true&latestby=cvpv

    // .query_param("q[][pacticipant]", "Foo")
    // .query_param("q[][version]", "1.2.4")
    // .query_param("q[][pacticipant]", "Bar")
    // .query_param("q[][latest]", "true")
    // .query_param("latestby", "cvpv");


  #[test]
  fn build_query_string_matrix_query_with_multiple_q_params_and_latestby_different_version() {
    // This test checks that the query string is built in the expected order for matrix queries
    // with multiple q[][param] and a latestby param, with a different version value.
    let mut query = HashMap::new();
    query.insert("q[][pacticipant]".to_string(), vec![Some("Foo".to_string()), Some("Bar".to_string())]);
    query.insert("q[][version]".to_string(), vec![Some("1.2.4".to_string())]);
    query.insert("q[][pacticipant]".to_string(), vec![Some("Bar".to_string())]);
    query.insert("q[][latest]".to_string(), vec![Some("true".to_string())]);
    query.insert("latestby".to_string(), vec![Some("cvpv".to_string())]);
    // The expected order is: q[][pacticipant]=Foo&q[][version]=1%2e2%2e4&q[][pacticipant]=Bar&q[][latest]=true&latestby=cvpv
    let result = build_query_string(query);
    assert_eq!(
      result,
      "q[][pacticipant]=Foo&q[][version]=1%2e2%2e4&q[][pacticipant]=Bar&q[][latest]=true&latestby=cvpv"
    );
  }
}
