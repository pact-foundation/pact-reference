//! Parser for the time portion of a date-time expression

use logos::{Lexer, Logos};

use crate::generators::datetime_expressions::{Adjustment, ClockHour, error, Operation, TimeBase, TimeOffsetType};

/// Struct storing the result of a parsed time expression
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedTimeExpression {
  /// Base to use to start the evaluation
  pub base: TimeBase,
  /// All the adjustments to make to the base
  pub adjustments: Vec<Adjustment<TimeOffsetType>>
}

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum TimeExpressionToken {
  #[token("now")]
  Now,

  #[token("midnight")]
  Midnight,

  #[token("noon")]
  Noon,

  #[token("o\'clock")]
  OClock,

  #[token("am")]
  Am,

  #[token("pm")]
  Pm,

  #[token("+")]
  Plus,

  #[token("-")]
  Minus,

  #[token("next")]
  Next,

  #[token("last")]
  Last,

  #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
  Digits(u64),

  #[regex("hour(s)?")]
  Hours,

  #[regex("minute(s)?")]
  Minutes,

  #[regex("second(s)?")]
  Seconds,

  #[regex("millisecond(s)?")]
  Milliseconds
}

impl TimeExpressionToken {
  fn is_base(&self) -> bool {
    match self {
      TimeExpressionToken::Now => true,
      TimeExpressionToken::Midnight => true,
      TimeExpressionToken::Noon => true,
      TimeExpressionToken::Digits(_) => true,
      _ => false
    }
  }

  fn is_op(&self) -> bool {
    match self {
      TimeExpressionToken::Plus => true,
      TimeExpressionToken::Minus => true,
      _ => false
    }
  }
}

// expression returns [ TimeBase timeBase = TimeBase.Now.INSTANCE, List<Adjustment<TimeOffsetType>> adj = new ArrayList<>() ] : ( base { $timeBase = $base.t; }
//     | op duration { if ($duration.d != null) $adj.add($duration.d.withOperation($op.o)); } ( op duration { if ($duration.d != null) $adj.add($duration.d.withOperation($op.o)); } )*
//     | base { $timeBase = $base.t; } ( op duration { if ($duration.d != null) $adj.add($duration.d.withOperation($op.o)); } )*
//     | 'next' offset { $adj.add(new Adjustment($offset.type, $offset.val, Operation.PLUS)); }
//     | 'next' offset { $adj.add(new Adjustment($offset.type, $offset.val, Operation.PLUS)); }  ( op duration {
//         if ($duration.d != null) $adj.add($duration.d.withOperation($op.o));
//     } )*
//     | 'last' offset { $adj.add(new Adjustment($offset.type, $offset.val, Operation.MINUS)); }
//     | 'last' offset { $adj.add(new Adjustment($offset.type, $offset.val, Operation.MINUS)); } ( op duration {
//         if ($duration.d != null) $adj.add($duration.d.withOperation($op.o));
//     } )*
//     ) EOF
//     ;
pub(crate) fn expression(lex: &mut Lexer<TimeExpressionToken>, exp: &str) -> anyhow::Result<ParsedTimeExpression> {
  let mut time_base = TimeBase::Now;
  let mut adj = vec![];

  if let Some(Ok(token)) = lex.next() {
    if token.is_base() {
      let (tb, next) = base(lex, exp, token)?;
      time_base = tb;
      if let Some(Ok(token)) = next.map(|next| Ok(next)).or_else(|| lex.next()) {
        if token.is_op() {
          let a = adjustments(lex, exp, token)?;
          adj.extend_from_slice(&*a);
        } else {
          return Err(error(exp, "+ or -", Some(lex.span().clone())));
        }
      }
    } else if token.is_op() {
      let a = adjustments(lex, exp, token)?;
      adj.extend_from_slice(&*a);
    } else if token == TimeExpressionToken::Next {
      let t = offset(lex, exp)?;
      adj.push(Adjustment {
        adjustment_type: t,
        value: 1,
        operation: Operation::PLUS
      });
      if let Some(Ok(token)) = lex.next() {
        if token.is_op() {
          let a = adjustments(lex, exp, token)?;
          adj.extend_from_slice(&*a);
        } else {
          return Err(error(exp, "+ or -", Some(lex.span().clone())));
        }
      }
    } else if token == TimeExpressionToken::Last {
      let t = offset(lex, exp)?;
      adj.push(Adjustment {
        adjustment_type: t,
        value: 1,
        operation: Operation::MINUS
      });
      if let Some(Ok(token)) = lex.next() {
        if token.is_op() {
          let a = adjustments(lex, exp, token)?;
          adj.extend_from_slice(&*a);
        } else {
          return Err(error(exp, "+ or -", Some(lex.span().clone())));
        }
      }
    } else {
      return Err(error(exp, "one of now, midnight, noon, 1-12 o'clock, +, -, next or last", Some(lex.span().clone())));
    }

    let span = lex.span();
    let remainder = lex.remainder().trim();
    if !remainder.is_empty() {
      Err(error(exp, "no more tokens", Some(span)))
    } else {
      Ok(ParsedTimeExpression {
        base: time_base,
        adjustments: adj
      })
    }
  } else {
    Err(error(exp, "one of now, midnight, noon, 1-12 o'clock, +, -, next or last", Some(lex.span().clone())))
  }
}

// base returns [ TimeBase t ] : 'now' { $t = TimeBase.Now.INSTANCE; }
//     | 'midnight' { $t = TimeBase.Midnight.INSTANCE; }
//     | 'noon' { $t = TimeBase.Noon.INSTANCE; }
//     | INT oclock { $t = TimeBase.of($INT.int, $oclock.h); }
//     ;
fn base(
  lex: &mut Lexer<TimeExpressionToken>,
  exp: &str,
  token: TimeExpressionToken
) -> anyhow::Result<(TimeBase, Option<TimeExpressionToken>)> {
  match token {
    TimeExpressionToken::Now => Ok((TimeBase::Now, None)),
    TimeExpressionToken::Midnight => Ok((TimeBase::Midnight, None)),
    TimeExpressionToken::Noon => Ok((TimeBase::Noon, None)),
    TimeExpressionToken::Digits(d) => {
      let span = lex.span();
      let (hour, next) = oclock(lex, exp)?;
      TimeBase::of(d, hour, exp, span)
        .map(|tb| (tb, next))
    }
    _ => Err(error(exp, "one of now, midnight, noon or number", Some(lex.span().clone())))
  }
}

// oclock returns [ ClockHour h ] : 'o\'clock' 'am' { $h = ClockHour.AM; }
//     | 'o\'clock' 'pm' { $h = ClockHour.PM; }
//     | 'o\'clock' { $h = ClockHour.NEXT; }
//     ;
fn oclock(
  lex: &mut Lexer<TimeExpressionToken>,
  exp: &str
) -> anyhow::Result<(ClockHour, Option<TimeExpressionToken>)> {
  if let Some(Ok(token)) = lex.next() {
    if token == TimeExpressionToken::OClock {
      let next = lex.next();
      if let Some(Ok(next)) = next {
        if next == TimeExpressionToken::Am {
          Ok((ClockHour::AM, None))
        } else if next == TimeExpressionToken::Pm {
          Ok((ClockHour::PM, None))
        } else {
          Ok((ClockHour::NEXT, Some(next)))
        }
      } else if next.is_none() {
        Ok((ClockHour::NEXT, None))
      } else {
        Err(error(exp, "am, pm, + or -", Some(lex.span().clone())))
      }
    } else {
      Err(error(exp, "o\'clock", Some(lex.span().clone())))
    }
  } else {
    Err(error(exp, "o\'clock", Some(lex.span().clone())))
  }
}

// adjustments: op duration ( op duration )*
fn adjustments(
  lex: &mut Lexer<TimeExpressionToken>,
  exp: &str,
  token: TimeExpressionToken
) -> anyhow::Result<Vec<Adjustment<TimeOffsetType>>> {
  let mut results = vec![];

  let mut token = token.clone();
  loop {
    if token.is_op() {
      let adj = adjustment(lex, exp, token)?;
      results.push(adj);
    } else {
      break
    }

    if let Some(Ok(t)) = lex.next() {
      // loop
      token = t.clone();
    } else {
      break
    }
  }

  Ok(results)
}

fn adjustment(
  lex: &mut Lexer<TimeExpressionToken>,
  exp: &str,
  token: TimeExpressionToken
) -> anyhow::Result<Adjustment<TimeOffsetType>> {
  let op = operation(lex, exp, token)?;
  let (adjustment_type, d) = duration(lex, exp)?;
  Ok(Adjustment {
    adjustment_type,
    value: d,
    operation: op
  })
}

// duration : INT durationType ;
fn duration(
  lex: &mut Lexer<TimeExpressionToken>,
  exp: &str
) -> anyhow::Result<(TimeOffsetType, u64)> {
  if let Some(Ok(TimeExpressionToken::Digits(n))) = lex.next() {
    let tot = duration_type(lex, exp)?;
    Ok((tot, n))
  } else {
    Err(error(exp, "an integer value", Some(lex.span().clone())))
  }
}

// durationType returns [ TimeOffsetType type ] : 'hour' { $type = TimeOffsetType.HOUR; }
//     | HOURS { $type = TimeOffsetType.HOUR; }
//     | 'minute' { $type = TimeOffsetType.MINUTE; }
//     | MINUTES { $type = TimeOffsetType.MINUTE; }
//     | 'second' { $type = TimeOffsetType.SECOND; }
//     | SECONDS { $type = TimeOffsetType.SECOND; }
//     | 'millisecond' { $type = TimeOffsetType.MILLISECOND; }
//     | MILLISECONDS { $type = TimeOffsetType.MILLISECOND; }
//     ;
fn duration_type(
  lex: &mut Lexer<TimeExpressionToken>,
  exp: &str
) -> anyhow::Result<TimeOffsetType> {
  if let Some(Ok(token)) = lex.next() {
    match token {
      TimeExpressionToken::Hours => Ok(TimeOffsetType::HOUR),
      TimeExpressionToken::Minutes => Ok(TimeOffsetType::MINUTE),
      TimeExpressionToken::Seconds => Ok(TimeOffsetType::SECOND),
      TimeExpressionToken::Milliseconds => Ok(TimeOffsetType::MILLISECOND),
      _ => Err(error(exp, "a duration type (hour(s), minute(s), etc.)", Some(lex.span().clone())))
    }
  } else {
    Err(error(exp, "a duration type (hour(s), minute(s), etc.)", Some(lex.span().clone())))
  }
}

// op returns [ Operation o ] : '+' { $o = Operation.PLUS; }
//     | '-' { $o = Operation.MINUS; }
//     ;
fn operation(
  lex: &mut Lexer<TimeExpressionToken>,
  exp: &str,
  token: TimeExpressionToken
) -> anyhow::Result<Operation> {
  match token {
    TimeExpressionToken::Plus => Ok(Operation::PLUS),
    TimeExpressionToken::Minus => Ok(Operation::MINUS),
    _ => Err(error(exp, "+ or -", Some(lex.span().clone())))
  }
}

// offset returns [ TimeOffsetType type, int val = 1 ] : 'hour' { $type = TimeOffsetType.HOUR; }
//     | 'minute' { $type = TimeOffsetType.MINUTE; }
//     | 'second' { $type = TimeOffsetType.SECOND; }
//     | 'millisecond' { $type = TimeOffsetType.MILLISECOND; }
//     ;
fn offset(
  lex: &mut Lexer<TimeExpressionToken>,
  exp: &str
) -> anyhow::Result<TimeOffsetType> {
  if let Some(Ok(token)) = lex.next() {
    match token {
      TimeExpressionToken::Hours => Ok(TimeOffsetType::HOUR),
      TimeExpressionToken::Minutes => Ok(TimeOffsetType::MINUTE),
      TimeExpressionToken::Seconds => Ok(TimeOffsetType::SECOND),
      TimeExpressionToken::Milliseconds => Ok(TimeOffsetType::MILLISECOND),
      _ => Err(error(exp, "an offset type (hour, minute, second, etc.)", Some(lex.span().clone())))
    }
  } else {
    Err(error(exp, "an offset type (hour, minute, second, etc.)", Some(lex.span().clone())))
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use logos::Logos;
  use pretty_assertions::assert_eq;
  use trim_margin::MarginTrimmable;

  use crate::generators::datetime_expressions::{Adjustment, Operation, TimeBase, TimeOffsetType};
  use crate::generators::time_expression_parser::ParsedTimeExpression;

  #[test]
  fn invalid_expression() {
    let mut lex = super::TimeExpressionToken::lexer("not valid");
    let result = super::expression(&mut lex, "not valid");
    assert_eq!(
      "|Error: Expected one of now, midnight, noon, 1-12 o'clock, +, -, next or last
       |   ╭─[ expression:1:1 ]
       |   │
       | 1 │ not valid
       |   │ ─┬ \u{0020}
       |   │  ╰── Expected one of now, midnight, noon, 1-12 o'clock, +, -, next or last here
       |───╯
       |
      ".trim_margin_with("|").unwrap(),
      result.unwrap_err().to_string()
    );

    let mut lex = super::TimeExpressionToken::lexer("44 o'clock");
    let result = super::expression(&mut lex, "44 o'clock");
    assert_eq!(
      "|Error: Expected hour 1 to 12
       |   ╭─[ expression:1:1 ]
       |   │
       | 1 │ 44 o'clock
       |   │ ─┬ \u{0020}
       |   │  ╰── Expected hour 1 to 12 here
       |───╯
       |
      ".trim_margin_with("|").unwrap(),
      result.unwrap_err().to_string()
    );

    let mut lex = super::TimeExpressionToken::lexer("now today not valid");
    let result = super::expression(&mut lex, "now today not valid");
    assert_eq!(
      "|Error: Expected no more tokens
       |   ╭─[ expression:1:5 ]
       |   │
       | 1 │ now today not valid
       |   │     ┬ \u{0020}
       |   │     ╰── Expected no more tokens here
       |───╯
       |
      ".trim_margin_with("|").unwrap(),
      result.unwrap_err().to_string()
    );
  }

  #[test]
  fn base_only() {
    let mut lex = super::TimeExpressionToken::lexer("now");
    expect!(super::expression(&mut lex, "now")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Now,
      adjustments: vec![]
    }));

    let mut lex = super::TimeExpressionToken::lexer("  midnight   ");
    expect!(super::expression(&mut lex, "  midnight   ")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Midnight,
      adjustments: vec![]
    }));

    let mut lex = super::TimeExpressionToken::lexer("noon");
    expect!(super::expression(&mut lex, "noon")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Noon,
      adjustments: vec![]
    }));

    let mut lex = super::TimeExpressionToken::lexer("1 o'clock");
    expect!(super::expression(&mut lex, "1 o'clock")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Next { hour: 1 },
      adjustments: vec![]
    }));
  }

  #[test]
  fn op_and_duration() {
    let mut lex = super::TimeExpressionToken::lexer("+1 hour");
    expect!(super::expression(&mut lex, "+1 hour")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Now,
      adjustments: vec![
        Adjustment {
          adjustment_type: TimeOffsetType::HOUR,
          value: 1,
          operation: Operation::PLUS
        }
      ]
    }));

    let mut lex = super::TimeExpressionToken::lexer("+ 2 hours - 1 second");
    expect!(super::expression(&mut lex, "+ 2 hours - 1 second")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Now,
      adjustments: vec![
        Adjustment {
          adjustment_type: TimeOffsetType::HOUR,
          value: 2,
          operation: Operation::PLUS
        },
        Adjustment {
          adjustment_type: TimeOffsetType::SECOND,
          value: 1,
          operation: Operation::MINUS
        }
      ]
    }));
  }

  #[test]
  fn base_and_op_and_duration() {
    let mut lex = super::TimeExpressionToken::lexer("midnight + 2 hours");
    expect!(super::expression(&mut lex, "midnight + 2 hours")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Midnight,
      adjustments: vec![
        Adjustment {
          adjustment_type: TimeOffsetType::HOUR,
          value: 2,
          operation: Operation::PLUS
        }
      ]
    }));

    let mut lex = super::TimeExpressionToken::lexer("midnight 2 week");
    let result = super::expression(&mut lex, "midnight 2 week");
    assert_eq!(
      "|Error: Expected + or -
       |   ╭─[ expression:1:10 ]
       |   │
       | 1 │ midnight 2 week
       |   │          ┬ \u{0020}
       |   │          ╰── Expected + or - here
       |───╯
       |
      ".trim_margin_with("|").unwrap(),
      result.unwrap_err().to_string()
    );
  }
}
