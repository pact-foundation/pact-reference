use std::iter::Peekable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathToken {
    Root,
    Field(String),
    Index(usize),
    Star,
    StarIndex
}

fn peek<I>(chars: &mut Peekable<I>) -> Option<(usize, char)> where I: Iterator<Item = (usize, char)> {
    chars.peek().map(|tup| (tup.0.clone(), tup.1.clone()))
}

// identifier -> a-zA-Z0-9+
fn identifier<I>(ch: char, chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &String)-> Result<(), String>
    where I: Iterator<Item = (usize, char)>  {
    let mut id = String::new();
    id.push(ch);
    let mut next_char = peek(chars);
    while next_char.is_some() {
        let ch = next_char.unwrap();
        if ch.1.is_alphabetic() || ch.1.is_numeric() || ch.1 == '_' {
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
fn path_identifier<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &String, index: usize)-> Result<(), String>
    where I: Iterator<Item = (usize, char)>  {
    match chars.next() {
        Some(ch) => match ch.1 {
            '*' => {
                tokens.push(PathToken::Star);
                Ok(())
            },
            c if c.is_alphabetic() || c.is_numeric() => {
                try!{ identifier(c, chars, tokens, path) }
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
fn string_path<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &String, index: usize)-> Result<(), String>
    where I: Iterator<Item = (usize, char)>  {
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
fn index_path<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &String)-> Result<(), String>
    where I: Iterator<Item = (usize, char)>  {
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
fn bracket_path<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &String, index: usize)-> Result<(), String>
    where I: Iterator<Item = (usize, char)>  {
    let mut ch = peek(chars);
    match ch {
        Some(c) => {
            if c.1 == '\'' {
                chars.next();
                try!{ string_path(chars, tokens, path, c.0) }
            } else if c.1.is_numeric() {
                try!{ index_path(chars, tokens, path) }
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
fn path_exp<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &String)-> Result<(), String>
    where I: Iterator<Item = (usize, char)> {
    let mut next_char = chars.next();
    while next_char.is_some() {
        let ch = next_char.unwrap();
        match ch.1 {
            '.' => try!{ path_identifier(chars, tokens, path, ch.0) },
            '[' => try!{ bracket_path(chars, tokens, path, ch.0) },
            _ => return Err(format!("Expected a \".\" or \"[\" instead of \"{}\" in path expression \"{}\" at index {}",
                ch.1, path, ch.0))
        }
        next_char = chars.next();
    }
    Ok(())
}

pub fn parse_path_exp(path: String) -> Result<Vec<PathToken>, String> {
    let mut tokens = vec![];

    // parse_path_exp -> $ path_exp | empty
    let mut chars = path.chars().enumerate().peekable();
    match chars.next() {
        Some(ch) => {
            if ch.1 == '$' {
                tokens.push(PathToken::Root);
                try!{ path_exp(&mut chars, &mut tokens, &path) }
                Ok(tokens)
            } else {
                Err(format!("Path expression \"{}\" does not start with a root marker \"$\"", path))
            }
        },
        None => Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;

    #[test]
    fn parse_path_exp_handles_empty_string() {
        expect!(parse_path_exp(s!(""))).to(be_ok().value(vec![]));
    }

    #[test]
    fn parse_path_exp_handles_root() {
        expect!(parse_path_exp(s!("$"))).to(be_ok().value(vec![PathToken::Root]));
    }

    #[test]
    fn parse_path_exp_handles_missing_root() {
        expect!(parse_path_exp(s!("adsjhaskjdh"))).to(
            be_err().value(s!("Path expression \"adsjhaskjdh\" does not start with a root marker \"$\"")));
    }

    #[test]
    fn parse_path_exp_handles_missing_path() {
        expect!(parse_path_exp(s!("$adsjhaskjdh"))).to(
            be_err().value(s!("Expected a \".\" or \"[\" instead of \"a\" in path expression \"$adsjhaskjdh\" at index 1")));
    }

    #[test]
    fn parse_path_exp_handles_missing_path_name() {
        expect!(parse_path_exp(s!("$."))).to(
            be_err().value(s!("Expected a path after \".\" in path expression \"$.\" at index 1")));
        expect!(parse_path_exp(s!("$.a.b.c."))).to(
            be_err().value(s!("Expected a path after \".\" in path expression \"$.a.b.c.\" at index 7")));
    }

    #[test]
    fn parse_path_exp_handles_invalid_identifiers() {
        expect!(parse_path_exp(s!("$.abc!"))).to(
            be_err().value(s!("\"!\" is not allowed in an identifier in path expression \"$.abc!\" at index 5")));
        expect!(parse_path_exp(s!("$.a.b.c.}"))).to(
            be_err().value(s!("Expected either a \"*\" or path identifier in path expression \"$.a.b.c.}\" at index 8")));
    }

    #[test]
    fn parse_path_exp_with_simple_identifiers() {
        expect!(parse_path_exp(s!("$.a"))).to(
            be_ok().value(vec![PathToken::Root, PathToken::Field(s!("a"))]));
        expect!(parse_path_exp(s!("$.a.b.c"))).to(
            be_ok().value(vec![PathToken::Root, PathToken::Field(s!("a")), PathToken::Field(s!("b")),
            PathToken::Field(s!("c"))]));
    }

    #[test]
    fn parse_path_exp_handles_underscores() {
        expect!(parse_path_exp(s!("$.user_id"))).to(
            be_ok().value(vec![PathToken::Root, PathToken::Field(s!("user_id"))])
        );
    }

    #[test]
    fn parse_path_exp_with_star_instead_of_identifiers() {
        expect!(parse_path_exp(s!("$.*"))).to(
            be_ok().value(vec![PathToken::Root, PathToken::Star]));
        expect!(parse_path_exp(s!("$.a.*.c"))).to(
            be_ok().value(vec![PathToken::Root, PathToken::Field(s!("a")), PathToken::Star,
            PathToken::Field(s!("c"))]));
    }

    #[test]
    fn parse_path_exp_with_bracket_notation() {
        expect!(parse_path_exp(s!("$['val1']"))).to(
            be_ok().value(vec![PathToken::Root, PathToken::Field(s!("val1"))]));
        expect!(parse_path_exp(s!("$.a['val@1.'].c"))).to(
            be_ok().value(vec![PathToken::Root, PathToken::Field(s!("a")), PathToken::Field(s!("val@1.")),
            PathToken::Field(s!("c"))]));
        expect!(parse_path_exp(s!("$.a[1].c"))).to(
            be_ok().value(vec![PathToken::Root, PathToken::Field(s!("a")), PathToken::Index(1),
            PathToken::Field(s!("c"))]));
        expect!(parse_path_exp(s!("$.a[*].c"))).to(
            be_ok().value(vec![PathToken::Root, PathToken::Field(s!("a")), PathToken::StarIndex,
            PathToken::Field(s!("c"))]));
    }

    #[test]
    fn parse_path_exp_with_invalid_bracket_notation() {
        expect!(parse_path_exp(s!("$["))).to(
            be_err().value(s!("Expected a \"'\" (single qoute) or a digit in path expression \"$[\" after index 1")));
        expect!(parse_path_exp(s!("$['"))).to(
            be_err().value(s!("Unterminated string in path expression \"$['\" at index 2")));
        expect!(parse_path_exp(s!("$['Unterminated string"))).to(
            be_err().value(s!("Unterminated string in path expression \"$['Unterminated string\" at index 21")));
        expect!(parse_path_exp(s!("$['']"))).to(
            be_err().value(s!("Empty strings are not allowed in path expression \"$['']\" at index 3")));
        expect!(parse_path_exp(s!("$['test'.b.c"))).to(
            be_err().value(s!("Unterminated brackets, found \".\" instead of \"]\" in path expression \"$['test'.b.c\" at index 8")));
        expect!(parse_path_exp(s!("$['test'"))).to(
            be_err().value(s!("Unterminated brackets in path expression \"$['test'\" at index 7")));
        expect!(parse_path_exp(s!("$['test']b.c"))).to(
            be_err().value(s!("Expected a \".\" or \"[\" instead of \"b\" in path expression \"$[\'test\']b.c\" at index 9")));
    }

    #[test]
    fn parse_path_exp_with_invalid_bracket_index_notation() {
        expect!(parse_path_exp(s!("$[dhghh]"))).to(
            be_err().value(s!("Indexes can only consist of numbers or a \"*\", found \"d\" instead in path expression \"$[dhghh]\" at index 2")));
        expect!(parse_path_exp(s!("$[12abc]"))).to(
            be_err().value(s!("Indexes can only consist of numbers or a \"*\", found \"a\" instead in path expression \"$[12abc]\" at index 4")));
        expect!(parse_path_exp(s!("$[]"))).to(
            be_err().value(s!("Empty bracket expressions are not allowed in path expression \"$[]\" at index 2")));
        expect!(parse_path_exp(s!("$[-1]"))).to(
            be_err().value(s!("Indexes can only consist of numbers or a \"*\", found \"-\" instead in path expression \"$[-1]\" at index 2")));
    }
}
