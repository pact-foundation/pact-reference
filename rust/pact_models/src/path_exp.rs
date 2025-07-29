//! Functions for dealing with path expressions

use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Write};
use std::hash::{Hash, Hasher};
use std::iter::Peekable;

use anyhow::anyhow;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use tracing::trace;

lazy_static! {
  // Only use "." syntax for things which are obvious identifiers.
  static ref IDENT: Regex = Regex::new(r#"^[_A-Za-z][_A-Za-z0-9]*$"#)
    .expect("could not parse IDENT regex");
  // Escape these characters when using string syntax.
  static ref ESCAPE: Regex = Regex::new(r#"\\|'"#)
    .expect("could not parse ESCAPE regex");
}

/// Struct to store path token
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum PathToken {
  /// Root token $
  #[default]
  Root,
  /// named field token
  Field(String),
  /// integer index token
  Index(usize),
  /// * token
  Star,
  /// * index token
  StarIndex
}

impl PathToken {
  /// If this path token is an index
  pub fn is_index(&self) -> bool {
    match self {
      PathToken::Index(_) => true,
      _ => false
    }
  }
}

impl Display for PathToken {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      PathToken::Root => write!(f, "$"),
      PathToken::Field(n) => write!(f, "{}", n),
      PathToken::Index(n) => write!(f, "{}", n),
      PathToken::Star => write!(f, "*"),
      PathToken::StarIndex => write!(f, "[*]")
    }
  }
}

fn matches_token(path_fragment: &str, path_token: &PathToken) -> usize {
  match path_token {
    PathToken::Root if path_fragment == "$" => 2,
    PathToken::Field(name) if path_fragment == name => 2,
    PathToken::Index(index) => match path_fragment.parse::<usize>() {
      Ok(i) if *index == i => 2,
      _ => 0
    },
    PathToken::StarIndex => if path_fragment == "[*]" {
      1
    } else {
      match path_fragment.parse::<usize>() {
        Ok(_) => 1,
        _ => 0
      }
    },
    PathToken::Star => 1,
    _ => 0
  }
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct DocPath {
  path_tokens: Vec<PathToken>,
  expr: String,
}

impl DocPath {
  /// Construct a new document path from the provided string path
  pub fn new(expr: impl Into<String>) -> anyhow::Result<Self> {
    let expr = expr.into();
    let path_tokens = parse_path_exp(&expr)
      .map_err(|e| anyhow!(e))?;
    Ok(Self {
      path_tokens,
      expr,
    })
  }

  /// Infallible construction for when the expression is statically known,
  /// intended for unit tests.
  ///
  /// Invalid expressions will still cause panics.
  pub fn new_unwrap(expr: &'static str) -> Self {
    Self::new(expr).unwrap()
  }

  /// Construct a new DocPath with an empty expression.
  ///
  /// Warning: do not call any of the `push_*` methods on this DocPath,
  /// as that would create an expression with invalid syntax
  /// (because it would be missing the Root token).
  pub fn empty() -> Self {
    Self {
      path_tokens: vec![],
      expr: "".into(),
    }
  }

  /// Construct a new DocPath with the Root token.
  pub fn root() -> Self {
    Self {
      path_tokens: vec![PathToken::Root],
      expr: "$".into(),
    }
  }

  /// Construct a new DocPath for `$.body`
  pub fn body() -> Self {
    Self {
      path_tokens: vec![PathToken::Root, PathToken::Field("body".to_string())],
      expr: "$.body".into(),
    }
  }

  /// Construct a new DocPath from a list of tokens
  pub fn from_tokens<I>(tokens: I) -> Self
    where I: IntoIterator<Item = PathToken> {
    let mut path = Self {
      path_tokens: tokens.into_iter().collect(),
      expr: "".into(),
    };
    path.expr = path.build_expr();
    path
  }

  /// Return the list of tokens that comprise this path.
  pub fn tokens(&self) -> &Vec<PathToken> {
    &self.path_tokens
  }

  /// Return the length, in parsed tokens.
  pub fn len(&self) -> usize {
    self.path_tokens.len()
  }

  /// Returns the last item in the path. Will panic if the path is empty.
  pub fn last(&self) -> Option<PathToken> {
    self.path_tokens.last().cloned()
  }

  /// Extract the string contents of the first Field token.
  /// For use with Header and Query DocPaths.
  pub fn first_field(&self) -> Option<&str> {
    for token in self.path_tokens.iter() {
      if let PathToken::Field(field) = token {
        return Some(field);
      }
    }
    return None;
  }

  /// Extract the string contents of the last Field token.
  pub fn last_field(&self) -> Option<&str> {
    for token in self.path_tokens.iter().rev() {
      if let PathToken::Field(field) = token {
        return Some(field);
      }
    }
    return None;
  }

  /// If this path is the root path (it has only one element, the root token `$`).
  pub fn is_root(&self) -> bool {
    &self.path_tokens == &[PathToken::Root]
  }

  /// The path is a wildcard path if it ends in a star (`*`)
  pub fn is_wildcard(&self) -> bool {
    self.path_tokens.last() == Some(&PathToken::Star)
  }

  /// Calculates the path weight for this path expression and a given path.
  /// Returns a tuple of the calculated weight and the number of path tokens matched.
  pub fn path_weight(&self, path: &[&str]) -> (usize, usize) {
    trace!("Calculating weight for path tokens '{:?}' and path '{:?}'",
           self.path_tokens, path);
    let weight = {
      if path.len() >= self.len() {
        (
          self.path_tokens.iter().zip(path.iter())
          .fold(1, |acc, (token, fragment)| acc * matches_token(fragment, token)),
          self.len()
        )
      } else {
        (0, self.len())
      }
    };
    trace!("Calculated weight {:?} for path '{}' and '{:?}'",
           weight, self, path);
    weight
  }

  /// If this path matches the given path. It will match if the calculated path weight is greater
  /// than zero (which means at least one token matched).
  pub fn matches_path(&self, path: &[&str]) -> bool {
    self.path_weight(path).0 > 0
  }

  /// If the path matches the given path (the calculated path weight is greater than zero) and
  /// both paths have the same length.
  pub fn matches_path_exactly(&self, path: &[&str]) -> bool {
     self.len() == path.len() && self.matches_path(path)
  }

  /// Creates a new path by cloning this one and pushing the string onto the end
  pub fn join(&self, part: impl Into<String>) -> Self {
    let part = part.into();
    let mut path = self.clone();
    if part == "*" {
      path.push_star();
    } else if part == "[*]" {
      path.push_star_index();
    } else if let Ok(index) = part.parse() {
      path.push_index(index);
    } else {
      path.push_field(part);
    }
    path
  }

  /// Creates a new path by cloning this one and joining the index onto the end. Paths that end
  /// with `*` will have the `*` replaced with the index.
  pub fn join_index(&self, index: usize) -> Self {
    let mut path = self.clone();
    match self.path_tokens.last() {
      Some(PathToken::Root) => { path.push_index(index); }
      Some(PathToken::Field(_)) => { path.push_index(index); }
      Some(PathToken::Index(_)) => { path.push_index(index); }
      Some(PathToken::Star) | Some(PathToken::StarIndex) => {
        if let Some(part) = path.path_tokens.last_mut() {
          *part = PathToken::Index(index);
          path.expr = path.build_expr();
        } else {
          path.push_index(index);
        }
      }
      None => { path.push_index(index); }
    }
    path
  }

  /// Creates a new path by cloning this one and joining the field onto the end. Paths that end
  /// with `*` will have the `*` replaced with the field.
  pub fn join_field<S: Into<String>>(&self, name: S) -> Self {
    let mut path = self.clone();
    match self.path_tokens.last() {
      Some(PathToken::Root) => { path.push_field(name.into()); }
      Some(PathToken::Field(_)) => { path.push_field(name.into()); }
      Some(PathToken::Index(_)) => { path.push_field(name.into()); }
      Some(PathToken::Star) => {
        if let Some(part) = path.path_tokens.last_mut() {
          *part = PathToken::Field(name.into());
          path.expr = path.build_expr();
        } else {
          path.push_field(name.into());
        }
      }
      Some(PathToken::StarIndex) => { path.push_field(name.into()); }
      None => { path.push_field(name.into()); }
    }
    path
  }

  /// Mutates this path by pushing a field value onto the end.
  pub fn push_field(&mut self, field: impl Into<String>) -> &mut Self {
    let field = field.into();
    write_obj_key_for_path(&mut self.expr, &field);
    self.path_tokens.push(PathToken::Field(field));
    self
  }

  /// Mutates this path by pushing an index value onto the end.
  pub fn push_index(&mut self, index: usize) -> &mut Self {
    self.path_tokens.push(PathToken::Index(index));
    // unwrap is safe, as write! is infallible for String
    let _ = write!(self.expr, "[{}]", index);
    self
  }

  /// Mutates this path by pushing a star value onto the end.
  pub fn push_star(&mut self) -> &mut Self {
    self.path_tokens.push(PathToken::Star);
    self.expr.push_str(".*");
    self
  }

  /// Mutates this path by pushing a star index value onto the end.
  pub fn push_star_index(&mut self) -> &mut Self {
    self.path_tokens.push(PathToken::StarIndex);
    self.expr.push_str("[*]");
    self
  }

  /// Mutates this path by pushing a path token onto the end.
  pub fn push(&mut self, path_token: PathToken) -> &mut Self {
    match &path_token {
      PathToken::Root => self.expr.push_str("$"),
      PathToken::Field(v) => {
        let s = &mut self.expr;
        write_obj_key_for_path(s, v.as_str())
      },
      PathToken::Index(i) => { let _ = write!(self.expr, "[{}]", i); },
      PathToken::Star => self.expr.push_str(".*"),
      PathToken::StarIndex => self.expr.push_str("[*]")
    };
    self.path_tokens.push(path_token);
    self
  }

  /// Mutates this path by pushing another path onto the end. Will drop the root marker from the
  /// other path
  pub fn push_path(&mut self, path: &DocPath) -> &mut Self {
    for token in &path.path_tokens {
      if token != &PathToken::Root {
        self.push(token.clone());
      }
    }
    self
  }

  /// Convert this path to a vector of strings
  pub fn to_vec(&self) -> Vec<String> {
    self.path_tokens.iter().map(|t| t.to_string()).collect()
  }

  /// Return the parent path from this one
  pub fn parent(&self) -> Option<Self> {
    if self.path_tokens.len() <= 1 {
      None
    } else {
      let mut vec = self.path_tokens.clone();
      vec.truncate(vec.len() - 1);
      let mut path = DocPath {
        path_tokens: vec,
        expr: "".to_string()
      };
      path.expr = path.build_expr();
      Some(path)
    }
  }

  fn build_expr(&self) -> String {
    let mut buffer = String::new();

    for token in &self.path_tokens {
      match token {
        PathToken::Root => buffer.push('$'),
        PathToken::Field(v) => {
          write_obj_key_for_path(&mut buffer, v.as_str());
        }
        PathToken::Index(i) => {
          let _ = write!(buffer, "[{}]", i);
        }
        PathToken::Star => {
          buffer.push('.');
          buffer.push('*');
        }
        PathToken::StarIndex => {
          buffer.push('[');
          buffer.push('*');
          buffer.push(']');
        }
      }
    }

    buffer
  }

  /// Returns a copy of this path will all parts lower case
  pub fn to_lower_case(&self) -> DocPath {
    DocPath {
      path_tokens: self.path_tokens.iter().map(|p| match p {
        PathToken::Field(f) => PathToken::Field(f.to_lowercase()),
        _ => p.clone()
      }).collect(),
      expr: self.expr.to_lowercase()
    }
  }

  /// Converts this path into a JSON pointer [RFC6901](https://datatracker.ietf.org/doc/html/rfc6901).
  pub fn as_json_pointer(&self) -> anyhow::Result<String> {
    let mut buffer = String::new();

    for token in &self.path_tokens {
      match token {
        PathToken::Root => {},
        PathToken::Field(v) => {
          let parsed = v.replace('~', "~0")
            .replace('/', "~1");
          let _ = write!(buffer, "/{}", parsed);
        }
        PathToken::Index(i) => {
          buffer.push('/');
          buffer.push_str(i.to_string().as_str());
        }
        PathToken::Star => {
          return Err(anyhow!("* can not be converted to a JSON pointer"));
        }
        PathToken::StarIndex => {
          return Err(anyhow!("* can not be converted to a JSON pointer"));
        }
      }
    }

    Ok(buffer)
  }

  /// If this path (as a string) ends with the given string
  pub fn ends_with(&self, suffix: &str) -> bool {
    self.expr.ends_with(suffix)
  }

  /// If this path (as a string) starts with the given string
  pub fn starts_with(&self, prefix: &str) -> bool {
    self.expr.starts_with(prefix)
  }

  /// Creates a new path with the last `n` parts removed.
  pub fn drop(&self, n: usize) -> Self {
    let vec = self.path_tokens.iter()
      .dropping_back(n)
      .cloned()
      .collect_vec();
    if vec.is_empty() {
      Self::root()
    } else {
      let mut path = DocPath {
        path_tokens: vec,
        expr: "".to_string()
      };
      path.expr = path.build_expr();
      path
    }
  }
}

/// Format a JSON object key for use in a JSON path expression. If we were
/// more concerned about performance, we might try to come up with a scheme
/// to minimize string allocation here.
fn write_obj_key_for_path(mut out: impl Write, key: &str) {
  // unwrap is safe, as write! is infallible for String
  let _ = if IDENT.is_match(key) {
    write!(out, ".{}", key)
  } else {
    write!(
      out,
      "['{}']",
      ESCAPE.replace_all(key, |caps: &Captures| format!(r#"\{}"#, &caps[0]))
    )
  };
}

#[cfg(test)]
fn obj_key_for_path(key: &str) -> String {
  let mut out = String::new();
  write_obj_key_for_path(&mut out, key);
  out
}

impl From<DocPath> for String {
  fn from(doc_path: DocPath) -> String {
    doc_path.expr
  }
}

impl From<&DocPath> for String {
  fn from(doc_path: &DocPath) -> String {
    doc_path.expr.clone()
  }
}

impl TryFrom<String> for DocPath {
  type Error = anyhow::Error;

  fn try_from(path: String) -> Result<Self, Self::Error> {
    DocPath::new(path)
  }
}

impl TryFrom<&String> for DocPath {
  type Error = anyhow::Error;

  fn try_from(path: &String) -> Result<Self, Self::Error> {
    DocPath::new(path)
  }
}

impl TryFrom<&str> for DocPath {
  type Error = anyhow::Error;

  fn try_from(path: &str) -> Result<Self, Self::Error> {
    DocPath::new(path)
  }
}

impl From<&DocPath>  for DocPath {
  fn from(value: &DocPath) -> Self {
    value.clone()
  }
}

impl PartialEq for DocPath {
  fn eq(&self, other: &Self) -> bool {
    self.expr == other.expr
  }
}

impl PartialOrd for DocPath {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for DocPath {
  fn cmp(&self, other: &Self) -> Ordering {
    self.expr.cmp(&other.expr)
  }
}

impl Hash for DocPath {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.expr.hash(state);
  }
}

impl Display for DocPath {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "{}", self.expr)
  }
}

fn peek<I>(chars: &mut Peekable<I>) -> Option<(usize, char)> where I: Iterator<Item = (usize, char)> {
  chars.peek().map(|tup| (tup.0.clone(), tup.1.clone()))
}

fn is_identifier_char(ch: char) -> bool {
  ch.is_alphabetic() || ch.is_numeric() || ch == '_' || ch == '-' || ch == ':' || ch == '#' || ch == '@'
}

// identifier -> a-zA-Z0-9+
fn identifier<I>(ch: char, chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &str) -> Result<(), String>
  where I: Iterator<Item=(usize, char)> {
  let mut id = String::new();
  id.push(ch);
  let mut next_char = peek(chars);
  while next_char.is_some() {
    let ch = next_char.unwrap();
    if is_identifier_char(ch.1) {
      chars.next();
      id.push(ch.1);
    } else if ch.1 == '.' || ch.1 == '\'' || ch.1 == '[' {
      break;
    } else {
      return Err(format!("\"{}\" is not allowed in an identifier in path expression \"{}\" at index {}",
                         ch.1, path, ch.0));
    }
    next_char = peek(chars);
  }
  tokens.push(PathToken::Field(id));
  Ok(())
}

// path_identifier -> identifier | *
fn path_identifier<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &str, index: usize) -> Result<(), String>
  where I: Iterator<Item=(usize, char)> {
  match chars.next() {
    Some(ch) => match ch.1 {
      '*' => {
        tokens.push(PathToken::Star);
        Ok(())
      },
      c if is_identifier_char(c) => {
        identifier(c, chars, tokens, path)?;
        Ok(())
      },
      _ => Err(format!("Expected either a \"*\" or path identifier in path expression \"{}\" at index {}",
                       path, ch.0))
    },
    None => Err(format!("Expected a path after \".\" in path expression \"{}\" at index {}",
                        path, index))
  }
}

// string_path -> [^']+
fn string_path<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &str, index: usize) -> Result<(), String>
  where I: Iterator<Item=(usize, char)> {
  let mut id = String::new();
  let mut next_char = peek(chars);
  if next_char.is_some() {
    chars.next();
    let mut ch = next_char.unwrap();
    next_char = peek(chars);
    while ch.1 != '\'' && next_char.is_some() {
      id.push(ch.1);
      chars.next();
      ch = next_char.unwrap();
      next_char = peek(chars);
    }
    if ch.1 == '\'' {
      if id.is_empty() {
        Err(format!("Empty strings are not allowed in path expression \"{}\" at index {}", path, ch.0))
      } else {
        tokens.push(PathToken::Field(id));
        Ok(())
      }
    } else {
      Err(format!("Unterminated string in path expression \"{}\" at index {}", path, ch.0))
    }
  } else {
    Err(format!("Unterminated string in path expression \"{}\" at index {}", path, index))
  }
}

// index_path -> [0-9]+
fn index_path<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &str) -> Result<(), String>
  where I: Iterator<Item=(usize, char)> {
  let mut id = String::new();
  let mut next_char = chars.next();
  id.push(next_char.unwrap().1);
  next_char = peek(chars);
  while next_char.is_some() {
    let ch = next_char.unwrap();
    if ch.1.is_numeric() {
      id.push(ch.1);
      chars.next();
    } else {
      break;
    }
    next_char = peek(chars);
  }

  if let Some(ch) = next_char {
    if ch.1 != ']' {
      return Err(format!("Indexes can only consist of numbers or a \"*\", found \"{}\" instead in path expression \"{}\" at index {}",
                         ch.1, path, ch.0))
    }
  }

  tokens.push(PathToken::Index(id.parse().unwrap()));
  Ok(())
}

// bracket_path -> (string_path | index | *) ]
fn bracket_path<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &str, index: usize) -> Result<(), String>
  where I: Iterator<Item=(usize, char)> {
  let mut ch = peek(chars);
  match ch {
    Some(c) => {
      if c.1 == '\'' {
        chars.next();
        string_path(chars, tokens, path, c.0)?
      } else if c.1.is_numeric() {
        index_path(chars, tokens, path)?
      } else if c.1 == '*' {
        chars.next();
        tokens.push(PathToken::StarIndex);
      } else if c.1 == ']' {
        return Err(format!("Empty bracket expressions are not allowed in path expression \"{}\" at index {}",
                           path, c.0));
      } else {
        return Err(format!("Indexes can only consist of numbers or a \"*\", found \"{}\" instead in path expression \"{}\" at index {}",
                           c.1, path, c.0));
      };
      ch = peek(chars);
      match ch {
        Some(c) => if c.1 != ']' {
          Err(format!("Unterminated brackets, found \"{}\" instead of \"]\" in path expression \"{}\" at index {}",
                      c.1, path, c.0))
        } else {
          chars.next();
          Ok(())
        },
        None => Err(format!("Unterminated brackets in path expression \"{}\" at index {}",
                            path, path.len() - 1))
      }
    },
    None => Err(format!("Expected a \"'\" (single qoute) or a digit in path expression \"{}\" after index {}",
                        path, index))
  }
}

// path_exp -> (dot-path | bracket-path)*
fn path_exp<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &str) -> Result<(), String>
  where I: Iterator<Item=(usize, char)> {
  let mut next_char = chars.next();
  while next_char.is_some() {
    let ch = next_char.unwrap();
    match ch.1 {
      '.' => path_identifier(chars, tokens, path, ch.0)?,
      '[' => bracket_path(chars, tokens, path, ch.0)?,
      _ => return Err(format!("Expected a \".\" or \"[\" instead of \"{}\" in path expression \"{}\" at index {}",
                              ch.1, path, ch.0))
    }
    next_char = chars.next();
  }
  Ok(())
}

pub fn parse_path_exp(path: &str) -> Result<Vec<PathToken>, String> {
  let mut tokens = vec![];

  // parse_path_exp -> $ path_exp | empty
  let mut chars = path.chars().enumerate().peekable();
  match chars.next() {
    Some(ch) => {
      match ch.1 {
        '$' => {
          tokens.push(PathToken::Root);
          path_exp(&mut chars, &mut tokens, path)?;
          Ok(tokens)
        }
        c if c.is_alphabetic() || c.is_numeric() => {
          tokens.push(PathToken::Root);
          identifier(c, &mut chars, &mut tokens, path)?;
          path_exp(&mut chars, &mut tokens, path)?;
          Ok(tokens)
        }
        _ => Err(format!("Path expression \"{}\" does not start with a root marker \"$\"", path))
      }
    }
    None => Ok(tokens)
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use rstest::rstest;
  use super::*;

  #[test]
  fn matches_token_test_with_root() {
    expect!(matches_token("$", &PathToken::Root)).to(be_equal_to(2));
    expect!(matches_token("path", &PathToken::Root)).to(be_equal_to(0));
    expect!(matches_token("*", &PathToken::Root)).to(be_equal_to(0));
  }

  #[test]
  fn matches_token_test_with_field() {
    expect!(matches_token("$", &PathToken::Field("path".to_string()))).to(be_equal_to(0));
    expect!(matches_token("path", &PathToken::Field("path".to_string()))).to(be_equal_to(2));
  }

  #[test]
  fn matches_token_test_with_index() {
    expect!(matches_token("$", &PathToken::Index(2))).to(be_equal_to(0));
    expect!(matches_token("path", &PathToken::Index(2))).to(be_equal_to(0));
    expect!(matches_token("*", &PathToken::Index(2))).to(be_equal_to(0));
    expect!(matches_token("1", &PathToken::Index(2))).to(be_equal_to(0));
    expect!(matches_token("2", &PathToken::Index(2))).to(be_equal_to(2));
  }

  #[test]
  fn matches_token_test_with_index_wildcard() {
    expect!(matches_token("$", &PathToken::StarIndex)).to(be_equal_to(0));
    expect!(matches_token("path", &PathToken::StarIndex)).to(be_equal_to(0));
    expect!(matches_token("*", &PathToken::StarIndex)).to(be_equal_to(0));
    expect!(matches_token("1", &PathToken::StarIndex)).to(be_equal_to(1));
  }

  #[test]
  fn matches_token_test_with_wildcard() {
    expect!(matches_token("$", &PathToken::Star)).to(be_equal_to(1));
    expect!(matches_token("path", &PathToken::Star)).to(be_equal_to(1));
    expect!(matches_token("*", &PathToken::Star)).to(be_equal_to(1));
    expect!(matches_token("1", &PathToken::Star)).to(be_equal_to(1));
  }

  #[test]
  fn docpath_empty() {
    expect!(DocPath::empty().path_tokens)
      .to(be_equal_to(DocPath::new_unwrap("").path_tokens));
  }

  #[test]
  fn docpath_root() {
    expect!(DocPath::root().path_tokens)
      .to(be_equal_to(DocPath::new_unwrap("$").path_tokens));
  }

  #[test]
  fn docpath_wildcard() {
    expect!(DocPath::new_unwrap("").is_wildcard()).to(be_equal_to(false));
    expect!(DocPath::new_unwrap("$.path").is_wildcard()).to(be_equal_to(false));
    expect!(DocPath::new_unwrap("$.*").is_wildcard()).to(be_equal_to(true));
    expect!(DocPath::new_unwrap("$.path.*").is_wildcard()).to(be_equal_to(true));
  }

  #[test]
  fn matches_path_matches_root_path_element() {
    expect!(DocPath::new_unwrap("$").matches_path(&vec!["$"])).to(be_true());
    expect!(DocPath::new_unwrap("$").matches_path(&vec![])).to(be_false());
  }

  #[test]
  fn matches_path_matches_field_name() {
    expect!(DocPath::new_unwrap("$.name").matches_path(&vec!["$", "name"])).to(be_true());
    expect!(DocPath::new_unwrap("$['name']").matches_path(&vec!["$", "name"])).to(be_true());
    expect!(DocPath::new_unwrap("$.name.other").matches_path(&vec!["$", "name", "other"])).to(be_true());
    expect!(DocPath::new_unwrap("$['name'].other").matches_path(&vec!["$", "name", "other"])).to(be_true());
    expect!(DocPath::new_unwrap("$.name").matches_path(&vec!["$", "other"])).to(be_false());
    expect!(DocPath::new_unwrap("$.name").matches_path(&vec!["$", "name", "other"])).to(be_true());
    expect!(DocPath::new_unwrap("$.other").matches_path(&vec!["$", "name", "other"])).to(be_false());
    expect!(DocPath::new_unwrap("$.name.other").matches_path(&vec!["$", "name"])).to(be_false());
  }

  #[test]
  fn matches_path_matches_array_indices() {
    expect!(DocPath::new_unwrap("$[0]").matches_path(&vec!["$", "0"])).to(be_true());
    expect!(DocPath::new_unwrap("$.name[1]").matches_path(&vec!["$", "name", "1"])).to(be_true());
    expect!(DocPath::new_unwrap("$.name").matches_path(&vec!["$", "0"])).to(be_false());
    expect!(DocPath::new_unwrap("$.name[1]").matches_path(&vec!["$", "name", "0"])).to(be_false());
    expect!(DocPath::new_unwrap("$[1].name").matches_path(&vec!["$", "name", "1"])).to(be_false());
  }

  #[test]
  fn matches_path_matches_with_wildcard() {
    expect!(DocPath::new_unwrap("$[*]").matches_path(&vec!["$", "0"])).to(be_true());
    expect!(DocPath::new_unwrap("$.*").matches_path(&vec!["$", "name"])).to(be_true());
    expect!(DocPath::new_unwrap("$.*.name").matches_path(&vec!["$", "some", "name"])).to(be_true());
    expect!(DocPath::new_unwrap("$.name[*]").matches_path(&vec!["$", "name", "0"])).to(be_true());
    expect!(DocPath::new_unwrap("$.name[*].name").matches_path(&vec!["$", "name", "1", "name"])).to(be_true());
    expect!(DocPath::new_unwrap("$[*]").matches_path(&vec!["$", "name"])).to(be_false());
  }

  #[test]
  fn parse_path_exp_handles_empty_string() {
    expect!(parse_path_exp("")).to(be_ok().value(vec![]));
  }

  #[test]
  fn parse_path_exp_handles_root() {
    expect!(parse_path_exp("$")).to(be_ok().value(vec![PathToken::Root]));
  }

  #[test]
  fn parse_path_exp_handles_missing_root() {
    expect!(parse_path_exp("adsjhaskjdh"))
      .to(be_ok().value(vec![PathToken::Root, PathToken::Field("adsjhaskjdh".to_string())]));
  }

  #[test]
  fn parse_path_exp_handles_missing_path() {
    expect!(parse_path_exp("$adsjhaskjdh")).to(
      be_err().value("Expected a \".\" or \"[\" instead of \"a\" in path expression \"$adsjhaskjdh\" at index 1".to_string()));
  }

  #[test]
  fn parse_path_exp_handles_missing_path_name() {
    expect!(parse_path_exp("$.")).to(
      be_err().value("Expected a path after \".\" in path expression \"$.\" at index 1".to_string()));
    expect!(parse_path_exp("$.a.b.c.")).to(
      be_err().value("Expected a path after \".\" in path expression \"$.a.b.c.\" at index 7".to_string()));
  }

  #[test]
  fn parse_path_exp_handles_invalid_identifiers() {
    expect!(parse_path_exp("$.abc!")).to(
      be_err().value("\"!\" is not allowed in an identifier in path expression \"$.abc!\" at index 5".to_string()));
    expect!(parse_path_exp("$.a.b.c.}")).to(
      be_err().value("Expected either a \"*\" or path identifier in path expression \"$.a.b.c.}\" at index 8".to_string()));
  }

  #[test]
  fn parse_path_exp_with_simple_identifiers() {
    expect!(parse_path_exp("$.a")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string())]));
    expect!(parse_path_exp("$.a.b.c")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string()), PathToken::Field("b".to_string()),
                         PathToken::Field("c".to_string())]));
    expect!(parse_path_exp("a.b.c")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string()), PathToken::Field("b".to_string()),
                         PathToken::Field("c".to_string())]));
  }

  #[test]
  fn parse_path_exp_handles_underscores_and_dashes() {
    expect!(parse_path_exp("$.user_id.user-id")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("user_id".to_string()),
                         PathToken::Field("user-id".to_string())])
    );
    expect!(parse_path_exp("$._id")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("_id".to_string())])
    );
    expect!(parse_path_exp("$.id:test")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("id:test".to_string())])
    );
  }

  #[test]
  fn parse_path_exp_handles_xml_names() {
    expect!(parse_path_exp("$.foo.@val")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("foo".to_string()),
                         PathToken::Field("@val".to_string())])
    );
    expect!(parse_path_exp("$.foo.#text")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("foo".to_string()),
                         PathToken::Field("#text".to_string())])
    );
    expect!(parse_path_exp("$.urn:ns:foo.urn:ns:something.#text")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("urn:ns:foo".to_string()),
                         PathToken::Field("urn:ns:something".to_string()), PathToken::Field("#text".to_string())])
    );
  }

  #[test]
  fn parse_path_exp_with_star_instead_of_identifiers() {
    expect!(parse_path_exp("$.*")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Star]));
    expect!(parse_path_exp("$.a.*.c")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string()), PathToken::Star,
                         PathToken::Field("c".to_string())]));
  }

  #[test]
  fn parse_path_exp_with_bracket_notation() {
    expect!(parse_path_exp("$['val1']")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("val1".to_string())]));
    expect!(parse_path_exp("$.a['val@1.'].c")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string()), PathToken::Field("val@1.".to_string()),
                         PathToken::Field("c".to_string())]));
    expect!(parse_path_exp("$.a[1].c")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string()), PathToken::Index(1),
                         PathToken::Field("c".to_string())]));
    expect!(parse_path_exp("$.a[*].c")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string()), PathToken::StarIndex,
                         PathToken::Field("c".to_string())]));
  }

  #[test]
  fn parse_path_exp_with_invalid_bracket_notation() {
    expect!(parse_path_exp("$[")).to(
      be_err().value("Expected a \"'\" (single qoute) or a digit in path expression \"$[\" after index 1".to_string()));
    expect!(parse_path_exp("$['")).to(
      be_err().value("Unterminated string in path expression \"$['\" at index 2".to_string()));
    expect!(parse_path_exp("$['Unterminated string")).to(
      be_err().value("Unterminated string in path expression \"$['Unterminated string\" at index 21".to_string()));
    expect!(parse_path_exp("$['']")).to(
      be_err().value("Empty strings are not allowed in path expression \"$['']\" at index 3".to_string()));
    expect!(parse_path_exp("$['test'.b.c")).to(
      be_err().value("Unterminated brackets, found \".\" instead of \"]\" in path expression \"$['test'.b.c\" at index 8".to_string()));
    expect!(parse_path_exp("$['test'")).to(
      be_err().value("Unterminated brackets in path expression \"$['test'\" at index 7".to_string()));
    expect!(parse_path_exp("$['test']b.c")).to(
      be_err().value("Expected a \".\" or \"[\" instead of \"b\" in path expression \"$[\'test\']b.c\" at index 9".to_string()));
  }

  #[test]
  fn parse_path_exp_with_invalid_bracket_index_notation() {
    expect!(parse_path_exp("$[dhghh]")).to(
      be_err().value("Indexes can only consist of numbers or a \"*\", found \"d\" instead in path expression \"$[dhghh]\" at index 2".to_string()));
    expect!(parse_path_exp("$[12abc]")).to(
      be_err().value("Indexes can only consist of numbers or a \"*\", found \"a\" instead in path expression \"$[12abc]\" at index 4".to_string()));
    expect!(parse_path_exp("$[]")).to(
      be_err().value("Empty bracket expressions are not allowed in path expression \"$[]\" at index 2".to_string()));
    expect!(parse_path_exp("$[-1]")).to(
      be_err().value("Indexes can only consist of numbers or a \"*\", found \"-\" instead in path expression \"$[-1]\" at index 2".to_string()));
  }

  #[test]
  fn obj_key_for_path_quotes_keys_when_necessary() {
    assert_eq!(obj_key_for_path("foo"), ".foo");
    assert_eq!(obj_key_for_path("_foo"), "._foo");
    assert_eq!(obj_key_for_path("["), "['[']");

    // I don't actually know how the JSON Path specification wants us to handle
    // these cases, but we need to _something_ to avoid panics or passing
    // `Result` around everywhere, so let's go with JavaScript string escape
    // syntax.
    assert_eq!(obj_key_for_path(r#"''"#), r#"['\'\'']"#);
    assert_eq!(obj_key_for_path(r#"a'"#), r#"['a\'']"#);
    assert_eq!(obj_key_for_path(r#"\"#), r#"['\\']"#);
  }

  #[test]
  fn path_join() {
    let something = DocPath::root().join("something");
    expect!(something.to_string()).to(be_equal_to("$.something"));
    expect!(DocPath::root().join("something else").to_string()).to(be_equal_to("$['something else']"));
    expect!(something.join("else").to_string()).to(be_equal_to("$.something.else"));
    expect!(something.join("*").to_string()).to(be_equal_to("$.something.*"));
    expect!(something.join("101").to_string()).to(be_equal_to("$.something[101]"));
  }

  #[test]
  fn path_push() {
    let mut root = DocPath::root();
    let something = root.push(PathToken::Field("something".to_string()));
    expect!(something.to_string()).to(be_equal_to("$.something"));
    expect!(DocPath::root().push(PathToken::Field("something else".to_string())).to_string())
      .to(be_equal_to("$['something else']"));
    expect!(something.push(PathToken::Field("else".to_string())).to_string())
      .to(be_equal_to("$.something.else"));
    expect!(something.push(PathToken::Star).to_string())
      .to(be_equal_to("$.something.else.*"));
    expect!(something.push(PathToken::Index(101)).to_string())
      .to(be_equal_to("$.something.else.*[101]"));
  }

  #[test]
  fn push_path() {
    let mut empty = DocPath::empty();
    let a = empty.push_field("a");
    let mut root = DocPath::root();
    let b = root.push_field("a").push_field("b");
    let mut root = DocPath::root();
    let c = root.push_field("a").push_field("b").push_field("se-token");
    expect!(DocPath::root().push_path(a).to_string())
      .to(be_equal_to("$.a"));
    expect!(DocPath::root().push_path(b).to_string())
      .to(be_equal_to("$.a.b"));
    expect!(DocPath::root().push_path(c).to_string())
      .to(be_equal_to("$.a.b['se-token']"));
  }

  #[test]
  fn build_expr() {
    let mut root = DocPath::root();
    expect!(root.build_expr()).to(be_equal_to("$"));
    let something = root.push(PathToken::Field("something".to_string()));
    expect!(something.build_expr()).to(be_equal_to("$.something"));
    expect!(DocPath::root().push(PathToken::Field("something else".to_string())).build_expr())
      .to(be_equal_to("$['something else']"));
    expect!(something.push(PathToken::Field("else".to_string())).build_expr())
      .to(be_equal_to("$.something.else"));
    expect!(something.push(PathToken::Star).build_expr())
      .to(be_equal_to("$.something.else.*"));
    expect!(something.push(PathToken::Index(101)).build_expr())
      .to(be_equal_to("$.something.else.*[101]"));
  }

  #[test]
  fn path_parent() {
    let something = DocPath::root().join("something");
    let something_else = something.join("else");
    let something_star = something.join("*");
    let something_escaped = something.join("e l s e");
    let something_escaped2 = something_escaped.join("two");
    let something_star_child = something_star.join("child");

    expect!(something.parent()).to(be_some().value(DocPath::root()));
    expect!(something_else.parent()).to(be_some().value(something.clone()));
    expect!(something_star.parent()).to(be_some().value(something.clone()));
    expect!(something_escaped.parent()).to(be_some().value(something.clone()));
    expect!(something_escaped2.parent()).to(be_some().value(something_escaped.clone()));
    expect!(something_star_child.parent()).to(be_some().value(something_star.clone()));

    expect!(DocPath::root().parent()).to(be_none());
    expect!(DocPath::empty().parent()).to(be_none());
  }

  #[test]
  fn as_json_pointer() {
    let root = DocPath::root();
    expect!(root.as_json_pointer().unwrap()).to(be_equal_to(""));

    let mut something = root.join("something");
    expect!(something.as_json_pointer().unwrap()).to(be_equal_to("/something"));

    let with_slash = something.join("a/b");
    expect!(with_slash.as_json_pointer().unwrap()).to(be_equal_to("/something/a~1b"));
    let with_tilde = something.join("m~n");
    expect!(with_tilde.as_json_pointer().unwrap()).to(be_equal_to("/something/m~0n"));

    let encoded = something.join("c%25d");
    expect!(encoded.as_json_pointer().unwrap()).to(be_equal_to("/something/c%25d"));

    expect!(DocPath::root().push(PathToken::Field("something else".to_string())).as_json_pointer().unwrap())
      .to(be_equal_to("/something else"));
    expect!(something.join("*").as_json_pointer()).to(be_err());
    expect!(something.push(PathToken::Index(101)).as_json_pointer().unwrap())
      .to(be_equal_to("/something/101"));
  }

  #[rstest(
    case("", "[0]"),
    case("$", "$[0]"),
    case("$.a", "$.a[0]"),
    case("$.a[1]", "$.a[1][0]"),
    case("$.a.*", "$.a[0]"),
    case("$.a[*]", "$.a[0]")
  )]
  fn join_index_test(#[case] base: &'static str, #[case] result: &str) {
    let base_path = DocPath::new_unwrap(base);
    let result_path = base_path.join_index(0);
    expect!(result_path.to_string()).to(be_equal_to(result));
  }
}
