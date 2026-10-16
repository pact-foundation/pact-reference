//! expression parser for generator expressions

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

use anyhow::anyhow;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::error;

use crate::json_utils::json_to_string;

/// Data type to cast to for provider state context values
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum DataType {
  /// String values
  STRING,
  /// Integer values
  INTEGER,
  /// Decimal values
  DECIMAL,
  /// Floating point values
  FLOAT,
  /// Don't convert values
  RAW,
  /// Boolean values
  BOOLEAN
}

impl DataType {
  /// Wraps the generated value in a DataValue
  pub fn wrap(&self, result: anyhow::Result<Value>) -> anyhow::Result<DataValue> {
    result.map(|val| DataValue { wrapped: val, data_type: *self })
  }
}

impl From<Value> for DataType {
  fn from(v: Value) -> Self {
    match v {
      Value::String(s) => match s.to_ascii_uppercase().as_str() {
        "STRING" => DataType::STRING,
        "INTEGER" => DataType::INTEGER,
        "DECIMAL" => DataType::DECIMAL,
        "FLOAT" => DataType::FLOAT,
        "BOOLEAN" => DataType::BOOLEAN,
        _ => DataType::RAW
      }
      _ => DataType::RAW
    }
  }
}

#[allow(clippy::from_over_into)]
impl Into<Value> for DataType {
  fn into(self) -> Value {
    match self {
      DataType::STRING => Value::String("STRING".to_string()),
      DataType::INTEGER => Value::String("INTEGER".to_string()),
      DataType::DECIMAL => Value::String("DECIMAL".to_string()),
      DataType::FLOAT => Value::String("FLOAT".to_string()),
      DataType::RAW => Value::String("RAW".to_string()),
      DataType::BOOLEAN => Value::String("BOOLEAN".to_string()),
    }
  }
}

#[allow(clippy::from_over_into)]
impl Into<Value> for &DataType {
  fn into(self) -> Value {
    (*self).into()
  }
}

/// Data Value container for a generated value
#[derive(Clone, Debug)]
pub struct DataValue {
  /// Original generated value
  pub wrapped: Value,
  /// Data type to cast it as
  pub data_type: DataType
}

impl DataValue {
  /// Convert this data value to JSON using the associated data type
  pub fn as_json(&self) -> anyhow::Result<Value> {
    match self.data_type {
      DataType::STRING => match &self.wrapped {
        Value::String(s) => Ok(Value::String(s.clone())),
        _ => Ok(Value::String(self.wrapped.to_string()))
      },
      DataType::INTEGER => match &self.wrapped {
        Value::Null => Ok(json!(0)),
        Value::Bool(b) => if *b {
          Ok(json!(1))
        } else {
          Ok(json!(0))
        }
        Value::Number(n) => if let Some(n) = n.as_u64() {
          Ok(json!(n))
        } else if let Some(n) = n.as_i64() {
          Ok(json!(n))
        } else if let Some(n) = n.as_f64() {
          Ok(json!(n as i64))
        } else {
          Err(anyhow!("Internal Error: Number is neither u64, i64, or f64"))
        }
        Value::String(s) => s.parse::<usize>().map(|val| json!(val))
          .map_err(|err| anyhow!("Number can not be generated from '{}' - {}", self.wrapped, err)),
        Value::Array(_) | Value::Object(_) => Err(anyhow!("Number can not be generated from '{}'", self.wrapped))
      },
      DataType::FLOAT | DataType::DECIMAL => match &self.wrapped {
        Value::Null => Ok(json!(0.0)),
        Value::Bool(b) => if *b {
          Ok(json!(1.0))
        } else {
          Ok(json!(0.0))
        }
        Value::Number(n) => if let Some(n) = n.as_u64() {
          Ok(json!(n as f64))
        } else if let Some(n) = n.as_i64() {
          Ok(json!(n as f64))
        } else if let Some(n) = n.as_f64() {
          Ok(json!(n))
        } else {
          Err(anyhow!("Internal Error: Number is neither u64, i64, or f64"))
        },
        Value::String(s) => s.parse::<f64>().map(|val| json!(val))
          .map_err(|err| anyhow!("Floating point number can not be generated from '{}' - {}", self.wrapped, err)),
        Value::Array(_) | Value::Object(_) => Err(anyhow!("Number can not be generated from '{}'", self.wrapped))
      },
      DataType::RAW => Ok(self.wrapped.clone()),
      DataType::BOOLEAN => match &self.wrapped {
        Value::Null => Ok(json!(false)),
        Value::Bool(b) => Ok(Value::Bool(*b)),
        Value::Number(n) => if let Some(n) = n.as_u64() {
          Ok(Value::Bool(n > 0))
        } else if let Some(n) = n.as_i64() {
          Ok(Value::Bool(n > 0))
        } else if let Some(n) = n.as_f64() {
          Ok(Value::Bool(n > 0.0))
        } else {
          Ok(Value::Bool(false))
        },
        Value::String(s) => s.parse::<bool>().map(|val| json!(val))
          .map_err(|err| anyhow!("Boolean can not be generated from '{}' - {}", self.wrapped, err)),
        Value::Array(_) | Value::Object(_) => Err(anyhow!("Boolean can not be generated from '{}'", self.wrapped))
      }
    }
  }
}

impl TryFrom<DataValue> for u16 {
  type Error = anyhow::Error;

  fn try_from(value: DataValue) -> Result<Self, Self::Error> {
    match &value.wrapped {
      Value::Null => Ok(0),
      Value::Bool(b) => if *b {
        Ok(1)
      } else {
        Ok(0)
      }
      Value::Number(n) => if let Some(n) = n.as_u64() {
        Ok(n as u16)
      } else if let Some(n) = n.as_i64() {
        Ok(n as u16)
      } else if let Some(n) = n.as_f64() {
        Ok(n as u16)
      } else {
        Ok(0)
      }
      Value::String(s) => match value.data_type {
        DataType::INTEGER | DataType::RAW | DataType::STRING => s.parse::<u16>()
          .map_err(|err| anyhow!("u16 can not be generated from '{}' - {}", s, err)),
        DataType::DECIMAL | DataType::FLOAT => s.parse::<f64>()
          .map(|val| val as u16)
          .map_err(|err| anyhow!("u16 can not be generated from '{}' - {}", s, err)),
        _ => Err(anyhow!("u16 can not be generated from {}", value.wrapped))
      },
      _ => Err(anyhow!("u16 can not be generated from {}", value.wrapped))
    }
  }
}

impl TryFrom<DataValue> for u64 {
  type Error = anyhow::Error;

  fn try_from(value: DataValue) -> Result<Self, Self::Error> {
    match &value.wrapped {
      Value::Null => Ok(0),
      Value::Bool(b) => if *b {
        Ok(1)
      } else {
        Ok(0)
      }
      Value::Number(n) => if let Some(n) = n.as_u64() {
        Ok(n)
      } else if let Some(n) = n.as_i64() {
        Ok(n as u64)
      } else if let Some(n) = n.as_f64() {
        Ok(n as u64)
      } else {
        Ok(0)
      }
      Value::String(s) => match value.data_type {
        DataType::INTEGER | DataType::RAW | DataType::STRING => s.parse::<u64>()
          .map_err(|err| anyhow!("u64 can not be generated from '{}' - {}", s, err)),
        DataType::DECIMAL | DataType::FLOAT => s.parse::<f64>()
          .map(|val| val as u64)
          .map_err(|err| anyhow!("u64 can not be generated from '{}' - {}", s, err)),
        _ => Err(anyhow!("u64 can not be generated from {}", value.wrapped))
      },
      _ => Err(anyhow!("u64 can not be generated from {}", value.wrapped))
    }
  }
}

impl TryFrom<DataValue> for i64 {
  type Error = anyhow::Error;

  fn try_from(value: DataValue) -> Result<Self, Self::Error> {
    match &value.wrapped {
      Value::Null => Ok(0),
      Value::Bool(b) => if *b {
        Ok(1)
      } else {
        Ok(0)
      }
      Value::Number(n) => if let Some(n) = n.as_u64() {
        Ok(n as i64)
      } else if let Some(n) = n.as_i64() {
        Ok(n)
      } else if let Some(n) = n.as_f64() {
        Ok(n as i64)
      } else {
        Ok(0)
      }
      Value::String(s) => match value.data_type {
        DataType::INTEGER | DataType::RAW | DataType::STRING => s.parse::<i64>()
          .map_err(|err| anyhow!("i64 can not be generated from '{}' - {}", s, err)),
        DataType::DECIMAL | DataType::FLOAT => s.parse::<f64>()
          .map(|val| val as i64)
          .map_err(|err| anyhow!("i64 can not be generated from '{}' - {}", s, err)),
        _ => Err(anyhow!("i64 can not be generated from {}", value.wrapped))
      },
      _ => Err(anyhow!("i64 can not be generated from {}", value.wrapped))
    }
  }
}

impl TryFrom<DataValue> for f64 {
  type Error = anyhow::Error;

  fn try_from(value: DataValue) -> Result<Self, Self::Error> {
    match &value.wrapped {
      Value::Null => Ok(0.0),
      Value::Bool(b) => if *b {
        Ok(1.0)
      } else {
        Ok(0.0)
      }
      Value::Number(n) => if let Some(n) = n.as_u64() {
        Ok(n as f64)
      } else if let Some(n) = n.as_i64() {
        Ok(n as f64)
      } else if let Some(n) = n.as_f64() {
        Ok(n)
      } else {
        Ok(0.0)
      }
      Value::String(s) => match value.data_type {
        DataType::INTEGER => s.parse::<i64>()
          .map(|val| val as f64)
          .map_err(|err| anyhow!("f64 can not be generated from '{}' - {}", s, err)),
        DataType::DECIMAL | DataType::FLOAT | DataType::RAW | DataType::STRING => s.parse::<f64>()
          .map_err(|err| anyhow!("f64 can not be generated from '{}' - {}", s, err)),
        _ => Err(anyhow!("f64 can not be generated from {}", value.wrapped))
      },
      _ => Err(anyhow!("f64 can not be generated from {}", value.wrapped))
    }
  }
}

impl TryFrom<DataValue> for bool {
  type Error = anyhow::Error;

  fn try_from(value: DataValue) -> Result<Self, Self::Error> {
    match &value.wrapped {
      Value::Null => Ok(false),
      Value::Bool(b) => Ok(*b),
      Value::Number(n) => if let Some(n) = n.as_u64() {
        Ok(n > 0)
      } else if let Some(n) = n.as_i64() {
        Ok(n > 0)
      } else if let Some(n) = n.as_f64() {
        Ok(n > 0.0)
      } else {
        Ok(false)
      }
      Value::String(s) => s.parse::<bool>()
        .map_err(|err| anyhow!("Boolean can not be generated from '{}' - {}", s, err)),
      _ => Err(anyhow!("Boolean can not be generated from {}", value.wrapped))
    }
  }
}

impl Display for DataValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self.data_type {
      DataType::STRING | DataType::RAW => write!(f, "{}", json_to_string(&self.wrapped)),
      DataType::INTEGER => match i64::try_from(self.clone()) {
        Ok(v) => write!(f, "{}", v),
        Err(err) => {
          error!("Failed to convert value = {}", err);
          Err(std::fmt::Error)
        }
      }
      DataType::FLOAT | DataType::DECIMAL => match f64::try_from(self.clone()) {
        Ok(v) => write!(f, "{}", v),
        Err(err) => {
          error!("Failed to convert value = {}", err);
          Err(std::fmt::Error)
        }
      }
      DataType::BOOLEAN => match bool::try_from(self.clone()) {
        Ok(v) => write!(f, "{}", v),
        Err(err) => {
          error!("Failed to convert value = {}", err);
          Err(std::fmt::Error)
        }
      }
    }
  }
}

/// Trait for resolvers of values
pub trait ValueResolver<T> {
  /// Resolves the value by looking it up in the context
  fn resolve_value(&self, name: &str) -> Option<T>;
}

/// Value resolver that looks a value up from a Map
#[derive(PartialEq, Debug, Clone)]
pub struct MapValueResolver<'a> {
  /// Map to resolve values from
  pub context: HashMap<&'a str, Value>
}

impl ValueResolver<String> for MapValueResolver<'_> {
  fn resolve_value(&self, name: &str) -> Option<String> {
    self.context.get(name).map(|val| json_to_string(val))
  }
}

impl ValueResolver<Value> for MapValueResolver<'_> {
  fn resolve_value(&self, name: &str) -> Option<Value> {
    self.context.get(name).cloned()
  }
}

/// If the String value contains any expression within it
pub fn contains_expressions(value: &str) -> bool {
  value.contains("${")
}

/// Parse the expressions and return the generated value
pub fn parse_expression(value: &str, value_resolver: &dyn ValueResolver<Value>) -> anyhow::Result<Value> {
  if contains_expressions(value) {
    replace_expressions(value, value_resolver)
  } else {
    Ok(json!(value))
  }
}

fn replace_expressions(value: &str, value_resolver: &dyn ValueResolver<Value>) -> anyhow::Result<Value> {
  let mut result = vec![];
  let mut buffer = value;
  let mut position = buffer.find("${");
  while let Some(index) = position {
    if index > 0 {
      result.push(json!(&buffer[0..index]));
    }
    let end_position = buffer.find('}')
      .ok_or_else(|| anyhow!("Missing closing brace in expression string '{}'", value))?;
    if end_position - index > 2 {
      let lookup_key = &buffer[(index + 2)..end_position];
      if let Some(lookup) = value_resolver.resolve_value(lookup_key) {
        result.push(lookup);
      } else {
        return Err(anyhow!("No value for '{}' found", lookup_key));
      }
    }
    buffer = &buffer[(end_position + 1)..];
    position = buffer.find("${");
  }
  if !buffer.is_empty() {
    result.push(json!(buffer));
  }

  if result.len() == 1 {
    Ok(result.first().unwrap().clone())
  } else {
    Ok(json!(result.iter().map(|val| json_to_string(val)).join("")))
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;

  use super::*;

  struct NullResolver;

  impl ValueResolver<Value> for NullResolver {
    fn resolve_value(&self, _: &str) -> Option<Value> {
      None
    }
  }

  #[test]
  fn does_not_modify_strings_with_no_expressions() {
    expect!(parse_expression("this is not an expression", &NullResolver)).to(be_ok().value(json!("this is not an expression")));
    expect!(parse_expression("", &NullResolver)).to(be_ok().value("".to_string()));
    expect!(parse_expression("looks like a $", &NullResolver)).to(be_ok().value("looks like a $".to_string()));
  }

  #[test]
  fn parse_expression_with_an_expression() {
    let resolver = MapValueResolver { context: hashmap!{ "a" => json!("A") } };
    expect!(parse_expression("${a}", &resolver)).to(be_ok().value("A".to_string()));
    expect!(parse_expression("${b}", &resolver)).to(be_err());
  }

  #[test]
  fn contains_expressions_test() {
    expect!(contains_expressions("${a}")).to(be_true());
    expect!(contains_expressions("$a")).to(be_false());
    expect!(contains_expressions("")).to(be_false());
    expect!(contains_expressions("this is not an expression")).to(be_false());
    expect!(contains_expressions("this ${is} an expression")).to(be_true());
  }

  #[test]
  fn returns_an_error_on_unterminated_expressions() {
    let result = parse_expression("invalid ${a expression", &NullResolver);
    expect!(result.as_ref()).to(be_err());
    expect!(result.unwrap_err().to_string())
      .to(be_equal_to("Missing closing brace in expression string \'invalid ${a expression\'".to_string()));
  }

  #[test]
  fn handles_empty_expression() {
    expect!(parse_expression("${}", &NullResolver)).to(be_ok().value("".to_string()));
    expect!(parse_expression("${} ${} ${}", &NullResolver)).to(be_ok().value("  ".to_string()));
  }

  #[test]
  fn replaces_the_expression_with_resolved_value() {
    let resolver = MapValueResolver { context: hashmap!{ "value" => json!("[value]") } };
    expect!(parse_expression("${value}", &resolver)).to(be_ok().value("[value]".to_string()));
    expect!(parse_expression(" ${value}", &resolver)).to(be_ok().value(" [value]".to_string()));
    expect!(parse_expression("${value} ", &resolver)).to(be_ok().value("[value] ".to_string()));
    expect!(parse_expression(" ${value} ", &resolver)).to(be_ok().value(" [value] ".to_string()));
    expect!(parse_expression(" ${value} ${value} ", &resolver)).to(be_ok().value(" [value] [value] ".to_string()));
    expect!(parse_expression("$${value}}", &resolver)).to(be_ok().value("$[value]}".to_string()));
  }

  #[test]
  fn keeps_the_type_of_simple_resolved_expressions() {
    let resolver = MapValueResolver { context: hashmap!{
      "value1" => json!("[value]"),
      "value2" => json!(100)
    } };
    expect!(parse_expression("${value1}", &resolver)).to(be_ok().value("[value]".to_string()));
    expect!(parse_expression("${value2}", &resolver)).to(be_ok().value(json!(100)));
    expect!(parse_expression("${value2} ", &resolver)).to(be_ok().value(json!("100 ")));
    expect!(parse_expression("${value1}/${value2}", &resolver)).to(be_ok().value(json!("[value]/100")));
  }

  #[test]
  fn with_a_defined_type_converts_the_expression_into_the_correct_type() {
    expect!(u16::try_from(DataValue { wrapped: json!("100"), data_type: DataType::RAW })).to(be_ok().value(100));
    expect!(u16::try_from(DataValue { wrapped: json!("100"), data_type: DataType::INTEGER })).to(be_ok().value(100));
    expect!(u16::try_from(DataValue { wrapped: json!("100.6"), data_type: DataType::INTEGER })).to(be_err());
    expect!(u16::try_from(DataValue { wrapped: json!("100"), data_type: DataType::FLOAT })).to(be_ok().value(100));
    expect!(u16::try_from(DataValue { wrapped: json!("100.6"), data_type: DataType::FLOAT })).to(be_ok().value(100));
    expect!(u16::try_from(DataValue { wrapped: json!("100"), data_type: DataType::STRING })).to(be_ok().value(100));
    expect!(u16::try_from(DataValue { wrapped: json!("string"), data_type: DataType::STRING })).to(be_err());
    expect!(u16::try_from(DataValue { wrapped: json!("true"), data_type: DataType::BOOLEAN })).to(be_err());

    expect!(u64::try_from(DataValue { wrapped: json!("100"), data_type: DataType::RAW })).to(be_ok().value(100));
    expect!(u64::try_from(DataValue { wrapped: json!("100"), data_type: DataType::INTEGER })).to(be_ok().value(100));
    expect!(u64::try_from(DataValue { wrapped: json!("100.6"), data_type: DataType::INTEGER })).to(be_err());
    expect!(u64::try_from(DataValue { wrapped: json!("100"), data_type: DataType::FLOAT })).to(be_ok().value(100));
    expect!(u64::try_from(DataValue { wrapped: json!("100.6"), data_type: DataType::FLOAT })).to(be_ok().value(100));
    expect!(u64::try_from(DataValue { wrapped: json!("100"), data_type: DataType::STRING })).to(be_ok().value(100));
    expect!(u64::try_from(DataValue { wrapped: json!("string"), data_type: DataType::STRING })).to(be_err());
    expect!(u64::try_from(DataValue { wrapped: json!("true"), data_type: DataType::BOOLEAN })).to(be_err());

    expect!(u64::try_from(DataValue { wrapped: json!(100), data_type: DataType::RAW })).to(be_ok().value(100));
    expect!(u64::try_from(DataValue { wrapped: json!(100), data_type: DataType::INTEGER })).to(be_ok().value(100));
    expect!(u64::try_from(DataValue { wrapped: json!(100.6), data_type: DataType::RAW })).to(be_ok().value(100));
    expect!(u64::try_from(DataValue { wrapped: json!(100.6), data_type: DataType::INTEGER })).to(be_ok().value(100));

    expect!(f64::try_from(DataValue { wrapped: json!("100"), data_type: DataType::RAW })).to(be_ok().value(100.0));
    expect!(f64::try_from(DataValue { wrapped: json!("100"), data_type: DataType::INTEGER })).to(be_ok().value(100.0));
    expect!(f64::try_from(DataValue { wrapped: json!("100.6"), data_type: DataType::INTEGER })).to(be_err());
    expect!(f64::try_from(DataValue { wrapped: json!("100"), data_type: DataType::FLOAT })).to(be_ok().value(100.0));
    expect!(f64::try_from(DataValue { wrapped: json!("100.6"), data_type: DataType::FLOAT })).to(be_ok().value(100.6));
    expect!(f64::try_from(DataValue { wrapped: json!("100"), data_type: DataType::STRING })).to(be_ok().value(100.0));
    expect!(f64::try_from(DataValue { wrapped: json!("string"), data_type: DataType::STRING })).to(be_err());
    expect!(f64::try_from(DataValue { wrapped: json!("true"), data_type: DataType::BOOLEAN })).to(be_err());

    expect!(f64::try_from(DataValue { wrapped: json!(100.6), data_type: DataType::RAW })).to(be_ok().value(100.6));
    expect!(f64::try_from(DataValue { wrapped: json!(100.6), data_type: DataType::INTEGER })).to(be_ok().value(100.6));
    expect!(f64::try_from(DataValue { wrapped: json!(100), data_type: DataType::RAW })).to(be_ok().value(100.0));
    expect!(f64::try_from(DataValue { wrapped: json!(100), data_type: DataType::STRING })).to(be_ok().value(100.0));

    expect!(DataValue { wrapped: json!("string"), data_type: DataType::RAW }.as_json()).to(be_ok().value(json!("string")));
    expect!(DataValue { wrapped: json!("string"), data_type: DataType::STRING }.as_json()).to(be_ok().value(json!("string")));
    expect!(DataValue { wrapped: json!("string"), data_type: DataType::INTEGER }.as_json()).to(be_err());
    expect!(DataValue { wrapped: json!("string"), data_type: DataType::FLOAT }.as_json()).to(be_err());
    expect!(DataValue { wrapped: json!("string"), data_type: DataType::DECIMAL }.as_json()).to(be_err());
    expect!(DataValue { wrapped: json!("string"), data_type: DataType::BOOLEAN }.as_json()).to(be_err());

    expect!(DataValue { wrapped: json!("100"), data_type: DataType::RAW }.as_json()).to(be_ok().value(json!("100")));
    expect!(DataValue { wrapped: json!(100), data_type: DataType::RAW }.as_json()).to(be_ok().value(json!(100)));
    expect!(DataValue { wrapped: json!("100"), data_type: DataType::STRING }.as_json()).to(be_ok().value(json!("100")));
    expect!(DataValue { wrapped: json!("100"), data_type: DataType::INTEGER }.as_json()).to(be_ok().value(json!(100)));
    expect!(DataValue { wrapped: json!(100), data_type: DataType::INTEGER }.as_json()).to(be_ok().value(json!(100)));
    expect!(DataValue { wrapped: json!("100"), data_type: DataType::FLOAT }.as_json()).to(be_ok().value(json!(100.0)));
    expect!(DataValue { wrapped: json!(100), data_type: DataType::FLOAT }.as_json()).to(be_ok().value(json!(100.0)));
    expect!(DataValue { wrapped: json!("100"), data_type: DataType::DECIMAL }.as_json()).to(be_ok().value(json!(100.0)));
    expect!(DataValue { wrapped: json!("100"), data_type: DataType::BOOLEAN }.as_json()).to(be_err());
    expect!(DataValue { wrapped: json!(100), data_type: DataType::BOOLEAN }.as_json()).to(be_ok().value(json!(true)));

    expect!(DataValue { wrapped: json!("100.5"), data_type: DataType::RAW }.as_json()).to(be_ok().value(json!("100.5")));
    expect!(DataValue { wrapped: json!(100.5), data_type: DataType::RAW }.as_json()).to(be_ok().value(json!(100.5)));
    expect!(DataValue { wrapped: json!("100.5"), data_type: DataType::STRING }.as_json()).to(be_ok().value(json!("100.5")));
    expect!(DataValue { wrapped: json!("100.5"), data_type: DataType::INTEGER }.as_json()).to(be_err());
    expect!(DataValue { wrapped: json!(100.5), data_type: DataType::INTEGER }.as_json()).to(be_ok().value(json!(100)));
    expect!(DataValue { wrapped: json!("100.5"), data_type: DataType::FLOAT }.as_json()).to(be_ok().value(json!(100.5)));
    expect!(DataValue { wrapped: json!(100.5), data_type: DataType::FLOAT }.as_json()).to(be_ok().value(json!(100.5)));
    expect!(DataValue { wrapped: json!("100.5"), data_type: DataType::DECIMAL }.as_json()).to(be_ok().value(json!(100.5)));
    expect!(DataValue { wrapped: json!("100.5"), data_type: DataType::BOOLEAN }.as_json()).to(be_err());

    expect!(DataValue { wrapped: json!("true"), data_type: DataType::RAW }.as_json()).to(be_ok().value(json!("true")));
    expect!(DataValue { wrapped: json!(true), data_type: DataType::RAW }.as_json()).to(be_ok().value(json!(true)));
    expect!(DataValue { wrapped: json!("true"), data_type: DataType::STRING }.as_json()).to(be_ok().value(json!("true")));
    expect!(DataValue { wrapped: json!(true), data_type: DataType::STRING }.as_json()).to(be_ok().value(json!("true")));
    expect!(DataValue { wrapped: json!("true"), data_type: DataType::INTEGER }.as_json()).to(be_err());
    expect!(DataValue { wrapped: json!(true), data_type: DataType::INTEGER }.as_json()).to(be_ok().value(json!(1)));
    expect!(DataValue { wrapped: json!("true"), data_type: DataType::FLOAT }.as_json()).to(be_err());
    expect!(DataValue { wrapped: json!("true"), data_type: DataType::DECIMAL }.as_json()).to(be_err());
    expect!(DataValue { wrapped: json!("true"), data_type: DataType::BOOLEAN }.as_json()).to(be_ok().value(json!(true)));
    expect!(DataValue { wrapped: json!(true), data_type: DataType::BOOLEAN }.as_json()).to(be_ok().value(json!(true)));
  }
}
