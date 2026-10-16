//! Parser for the date portion of a date-time expression

use logos::{Lexer, Logos};

use crate::generators::datetime_expressions::{Adjustment, error, Operation};
use crate::generators::datetime_expressions::DateBase;
use crate::generators::datetime_expressions::DateOffsetType;

/// Struct storing the result of a parsed date expression
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedDateExpression {
  /// Base to use to start the evaluation
  pub base: DateBase,
  /// All the adjustments to make to the base
  pub adjustments: Vec<Adjustment<DateOffsetType>>
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum DateExpressionToken {
  #[token("now")]
  Now,

  #[token("today")]
  Today,

  #[token("yesterday")]
  Yesterday,

  #[token("tomorrow")]
  Tomorrow,

  #[token("+")]
  Plus,

  #[token("-")]
  Minus,

  #[token("next")]
  Next,

  #[token("last")]
  Last,

  #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
  Int(u64),

  #[token("days")]
  Days,

  #[token("day")]
  Day,

  #[token("weeks")]
  Weeks,

  #[token("week")]
  Week,

  #[token("months")]
  Months,

  #[token("month")]
  Month,

  #[token("years")]
  Years,

  #[token("year")]
  Year,

  #[token("fortnight")]
  Fortnight,

  #[regex("mon(day)?")]
  Monday,

  #[regex("tues(day)?")]
  Tuesday,

  #[regex("wed(nesday)?")]
  Wednesday,

  #[regex("thurs(day)?")]
  Thursday,

  #[regex("fri(day)?")]
  Friday,

  #[regex("sat(urday)?")]
  Saturday,

  #[regex("sun(day)?")]
  Sunday,

  #[regex("jan(uary)?")]
  January,

  #[regex("feb(ruary)?")]
  February,

  #[regex("mar(ch)?")]
  March,

  #[regex("apr(il)?")]
  April,

  #[token("may")]
  May,

  #[regex("jun(e)?")]
  June,

  #[regex("jul(y)?")]
  July,

  #[regex("aug(ust)?")]
  August,

  #[regex("sep(tember)?")]
  September,

  #[regex("oct(ober)?")]
  October,

  #[regex("nov(ember)?")]
  November,

  #[regex("dec(ember)?")]
  December
}

impl DateExpressionToken {
  fn is_base(&self) -> bool {
    match self {
      DateExpressionToken::Now => true,
      DateExpressionToken::Today => true,
      DateExpressionToken::Tomorrow => true,
      DateExpressionToken::Yesterday => true,
      _ => false
    }
  }

  fn is_op(&self) -> bool {
    match self {
      DateExpressionToken::Plus => true,
      DateExpressionToken::Minus => true,
      _ => false
    }
  }
}

// expression returns [ DateBase dateBase = DateBase.NOW, List<Adjustment<DateOffsetType>> adj = new ArrayList<>() ] : ( base { $dateBase = $base.t; }
//     | op duration { if ($duration.d != null) $adj.add($duration.d.withOperation($op.o)); } ( op duration { if ($duration.d != null) $adj.add($duration.d.withOperation($op.o)); } )*
//     | base { $dateBase = $base.t; } ( op duration { if ($duration.d != null) $adj.add($duration.d.withOperation($op.o)); } )*
//     | 'next' offset { $adj.add(new Adjustment($offset.type, $offset.val, Operation.PLUS)); }
//     | 'next' offset { $adj.add(new Adjustment($offset.type, $offset.val, Operation.PLUS)); }  op duration {
//         if ($duration.d != null) $adj.add($duration.d.withOperation($op.o));
//     }
//     | 'last' offset { $adj.add(new Adjustment($offset.type, $offset.val, Operation.MINUS)); }
//     | 'last' offset { $adj.add(new Adjustment($offset.type, $offset.val, Operation.MINUS)); } (op duration {
//         if ($duration.d != null) $adj.add($duration.d.withOperation($op.o));
//     })*
//     ) EOF
//     ;
pub(crate) fn expression<'a>(lex: &mut Lexer<DateExpressionToken>, exp: &str) -> anyhow::Result<ParsedDateExpression> {
  let mut date_base = DateBase::NOW;
  let mut adj = vec![];

  if let Some(Ok(token)) = lex.next() {
    if token.is_base() {
      date_base = base(lex, exp, &token)?;
      if let Some(Ok(token)) = lex.next() {
        if token.is_op() {
          adj.extend_from_slice(&*adjustments(lex, exp, &token)?);
        } else {
          return Err(error(exp, "+ or -", Some(lex.span().clone())));
        }
      }
    } else if token.is_op() {
      adj.extend_from_slice(&*adjustments(lex, exp, &token)?);
    } else if token == DateExpressionToken::Next {
      let (t, v) = offset(lex, exp)?;
      adj.push(Adjustment {
        adjustment_type: t,
        value: v,
        operation: Operation::PLUS
      });
      if let Some(Ok(token)) = lex.next() {
        if token.is_op() {
          adj.extend_from_slice(&*adjustments(lex, exp, &token)?);
        } else {
          return Err(error(exp, "+ or -", Some(lex.span().clone())));
        }
      }
    } else if token == DateExpressionToken::Last {
      let (t, v) = offset(lex, exp)?;
      adj.push(Adjustment {
        adjustment_type: t,
        value: v,
        operation: Operation::MINUS
      });
      if let Some(Ok(token)) = lex.next() {
        if token.is_op() {
          adj.extend_from_slice(&*adjustments(lex, exp, &token)?);
        } else {
          return Err(error(exp, "+ or -", Some(lex.span().clone())));
        }
      }
    } else {
      return Err(error(exp, "one of now, today, yesterday, tomorrow, +, -, next or last", Some(lex.span().clone())));
    }

    let remainder = lex.remainder().trim();
    if !remainder.is_empty() {
      Err(error(exp, "no more tokens", Some(lex.span().clone())))
    } else {
      Ok(ParsedDateExpression {
        base: date_base,
        adjustments: adj
      })
    }
  } else {
    Err(error(exp, "one of now, today, yesterday, tomorrow, +, -, next or last", Some(lex.span().clone())))
  }
}

// op duration { if ($duration.d != null) $adj.add($duration.d.withOperation($op.o)); } ( op duration { if ($duration.d != null) $adj.add($duration.d.withOperation($op.o)); } )*
fn adjustments(lex: &mut Lexer<DateExpressionToken>, exp: &str, token: &DateExpressionToken) -> anyhow::Result<Vec<Adjustment<DateOffsetType>>> {
  let mut results = vec![];

  results.push(adjustment(lex, exp, token)?);
  while let Some(Ok(token)) = lex.next() {
    if token.is_op() {
      results.push(adjustment(lex, exp, &token)?);
    } else {
      break
    }
  }

  Ok(results)
}

fn adjustment(lex: &mut Lexer<DateExpressionToken>, exp: &str, token: &DateExpressionToken) -> anyhow::Result<Adjustment<DateOffsetType>> {
  let op = operation(lex, exp, token)?;
  let (adjustment_type, d) = duration(lex, exp)?;
  Ok(Adjustment {
    adjustment_type,
    value: d,
    operation: op
  })
}

// base returns [ DateBase t ] : 'now' { $t = DateBase.NOW; }
//     | 'today' { $t = DateBase.TODAY; }
//     | 'yesterday' { $t = DateBase.YESTERDAY; }
//     | 'tomorrow' { $t = DateBase.TOMORROW; }
//     ;
fn base(lex: &mut Lexer<DateExpressionToken>, exp: &str, token: &DateExpressionToken) -> anyhow::Result<DateBase> {
  match token {
    DateExpressionToken::Now => Ok(DateBase::NOW),
    DateExpressionToken::Today => Ok(DateBase::TODAY),
    DateExpressionToken::Tomorrow => Ok(DateBase::TOMORROW),
    DateExpressionToken::Yesterday => Ok(DateBase::YESTERDAY),
    _ => Err(error(exp, "one of now, today, yesterday or tomorrow", Some(lex.span().clone())))
  }
}

// op returns [ Operation o ] : '+' { $o = Operation.PLUS; }
//     | '-' { $o = Operation.MINUS; }
//     ;
fn operation(lex: &mut Lexer<DateExpressionToken>, exp: &str, token: &DateExpressionToken) -> anyhow::Result<Operation> {
  match token {
    DateExpressionToken::Plus => Ok(Operation::PLUS),
    DateExpressionToken::Minus => Ok(Operation::MINUS),
    _ => Err(error(exp, "+ or -", Some(lex.span().clone())))
  }
}

// duration : INT durationType
fn duration(lex: &mut Lexer<DateExpressionToken>, exp: &str) -> anyhow::Result<(DateOffsetType, u64)> {
  if let Some(Ok(DateExpressionToken::Int(n))) = lex.next() {
    let d = duration_type(lex, exp)?;
    Ok((d, n))
  } else {
    Err(error(exp, "an integer value", Some(lex.span().clone())))
  }
}

// durationType returns [ DateOffsetType type ] : 'day' { $type = DateOffsetType.DAY; }
//     | DAYS { $type = DateOffsetType.DAY; }
//     | 'week' { $type = DateOffsetType.WEEK; }
//     | WEEKS { $type = DateOffsetType.WEEK; }
//     | 'month' { $type = DateOffsetType.MONTH; }
//     | MONTHS { $type = DateOffsetType.MONTH; }
//     | 'year' { $type = DateOffsetType.YEAR; }
//     | YEARS { $type = DateOffsetType.YEAR; }
//     ;
fn duration_type(lex: &mut Lexer<DateExpressionToken>, exp: &str) -> anyhow::Result<DateOffsetType> {
  if let Some(Ok(token)) = lex.next() {
    match token {
      DateExpressionToken::Day => Ok(DateOffsetType::DAY),
      DateExpressionToken::Days => Ok(DateOffsetType::DAY),
      DateExpressionToken::Week => Ok(DateOffsetType::WEEK),
      DateExpressionToken::Weeks => Ok(DateOffsetType::WEEK),
      DateExpressionToken::Month => Ok(DateOffsetType::MONTH),
      DateExpressionToken::Months => Ok(DateOffsetType::MONTH),
      DateExpressionToken::Year => Ok(DateOffsetType::YEAR),
      DateExpressionToken::Years => Ok(DateOffsetType::YEAR),
      _ => Err(error(exp, "a duration type (day(s), week(s), etc.)", Some(lex.span().clone())))
    }
  } else {
    Err(error(exp, "a duration type (day(s), week(s), etc.)", Some(lex.span().clone())))
  }
}

// offset returns [ DateOffsetType type, int val = 1 ] : 'day' { $type = DateOffsetType.DAY; }
//     | 'week' { $type = DateOffsetType.WEEK; }
//     | 'month' { $type = DateOffsetType.MONTH; }
//     | 'year' { $type = DateOffsetType.YEAR; }
//     | 'fortnight' { $type = DateOffsetType.WEEK; $val = 2; }
//     | 'monday' { $type = DateOffsetType.MONDAY; }
//     | 'mon' { $type = DateOffsetType.MONDAY; }
//     | 'tuesday' { $type = DateOffsetType.TUESDAY; }
//     | 'tues' { $type = DateOffsetType.TUESDAY; }
//     | 'wednesday' { $type = DateOffsetType.WEDNESDAY; }
//     | 'wed' { $type = DateOffsetType.WEDNESDAY; }
//     | 'thursday' { $type = DateOffsetType.THURSDAY; }
//     | 'thurs' { $type = DateOffsetType.THURSDAY; }
//     | 'friday' { $type = DateOffsetType.FRIDAY; }
//     | 'fri' { $type = DateOffsetType.FRIDAY; }
//     | 'saturday' { $type = DateOffsetType.SATURDAY; }
//     | 'sat' { $type = DateOffsetType.SATURDAY; }
//     | 'sunday' { $type = DateOffsetType.SUNDAY; }
//     | 'sun' { $type = DateOffsetType.SUNDAY; }
//     | 'january' { $type = DateOffsetType.JAN; }
//     | 'jan' { $type = DateOffsetType.JAN; }
//     | 'february' { $type = DateOffsetType.FEB; }
//     | 'feb' { $type = DateOffsetType.FEB; }
//     | 'march' { $type = DateOffsetType.MAR; }
//     | 'mar' { $type = DateOffsetType.MAR; }
//     | 'april' { $type = DateOffsetType.APR; }
//     | 'apr' { $type = DateOffsetType.APR; }
//     | 'may' { $type = DateOffsetType.MAY; }
//     | 'june' { $type = DateOffsetType.JUNE; }
//     | 'jun' { $type = DateOffsetType.JUNE; }
//     | 'july' { $type = DateOffsetType.JULY; }
//     | 'jul' { $type = DateOffsetType.JULY; }
//     | 'august' { $type = DateOffsetType.AUG; }
//     | 'aug' { $type = DateOffsetType.AUG; }
//     | 'september' { $type = DateOffsetType.SEP; }
//     | 'sep' { $type = DateOffsetType.SEP; }
//     | 'october' { $type = DateOffsetType.OCT; }
//     | 'oct' { $type = DateOffsetType.OCT; }
//     | 'november' { $type = DateOffsetType.NOV; }
//     | 'nov' { $type = DateOffsetType.NOV; }
//     | 'december' { $type = DateOffsetType.DEC; }
//     | 'dec' { $type = DateOffsetType.DEC; }
//     ;
fn offset(lex: &mut Lexer<DateExpressionToken>, exp: &str) -> anyhow::Result<(DateOffsetType, u64)> {
  if let Some(Ok(token)) = lex.next() {
    match token {
      DateExpressionToken::Day => Ok((DateOffsetType::DAY, 1)),
      DateExpressionToken::Week => Ok((DateOffsetType::WEEK, 1)),
      DateExpressionToken::Month => Ok((DateOffsetType::MONTH, 1)),
      DateExpressionToken::Year => Ok((DateOffsetType::YEAR, 1)),
      DateExpressionToken::Fortnight => Ok((DateOffsetType::WEEK, 2)),
      DateExpressionToken::Monday => Ok((DateOffsetType::MONDAY, 1)),
      DateExpressionToken::Tuesday => Ok((DateOffsetType::TUESDAY, 1)),
      DateExpressionToken::Wednesday => Ok((DateOffsetType::WEDNESDAY, 1)),
      DateExpressionToken::Thursday => Ok((DateOffsetType::THURSDAY, 1)),
      DateExpressionToken::Friday => Ok((DateOffsetType::FRIDAY, 1)),
      DateExpressionToken::Saturday => Ok((DateOffsetType::SATURDAY, 1)),
      DateExpressionToken::Sunday => Ok((DateOffsetType::SUNDAY, 1)),
      DateExpressionToken::January => Ok((DateOffsetType::JAN, 1)),
      DateExpressionToken::February => Ok((DateOffsetType::FEB, 1)),
      DateExpressionToken::March => Ok((DateOffsetType::MAR, 1)),
      DateExpressionToken::April => Ok((DateOffsetType::APR, 1)),
      DateExpressionToken::May => Ok((DateOffsetType::MAY, 1)),
      DateExpressionToken::June => Ok((DateOffsetType::JUNE, 1)),
      DateExpressionToken::July => Ok((DateOffsetType::JULY, 1)),
      DateExpressionToken::August => Ok((DateOffsetType::AUG, 1)),
      DateExpressionToken::September => Ok((DateOffsetType::SEP, 1)),
      DateExpressionToken::October => Ok((DateOffsetType::OCT, 1)),
      DateExpressionToken::November => Ok((DateOffsetType::NOV, 1)),
      DateExpressionToken::December => Ok((DateOffsetType::DEC, 1)),
      _ => Err(error(exp, "an offset type (month, week, tuesday, february, etc.)", Some(lex.span().clone())))
    }
  } else {
    Err(error(exp, "an offset type (month, week, tuesday, february, etc.)", Some(lex.span().clone())))
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use logos::Logos;
  use pretty_assertions::assert_eq;
  use trim_margin::MarginTrimmable;

  use crate::generators::date_expression_parser::ParsedDateExpression;
  use crate::generators::datetime_expressions::{Adjustment, DateBase, DateOffsetType, Operation};

  #[test]
  fn invalid_expression() {
    let mut lex = super::DateExpressionToken::lexer("not valid");
    let result = super::expression(&mut lex, "not valid");
    assert_eq!(
      "|Error: Expected one of now, today, yesterday, tomorrow, +, -, next or last
          |   ╭─[ expression:1:1 ]
          |   │
          | 1 │ not valid
          |   │ ─┬ \u{0020}
          |   │  ╰── Expected one of now, today, yesterday, tomorrow, +, -, next or last here
          |───╯
          |
          ".trim_margin_with("|").unwrap(),
      result.unwrap_err().to_string()
    );

    let mut lex = super::DateExpressionToken::lexer("now today not valid");
    let result = super::expression(&mut lex, "now today not valid");
    assert_eq!(
      "|Error: Expected + or -
          |   ╭─[ expression:1:5 ]
          |   │
          | 1 │ now today not valid
          |   │     ──┬── \u{0020}
          |   │       ╰──── Expected + or - here
          |───╯
          |
          ".trim_margin_with("|").unwrap(),
      result.unwrap_err().to_string()
    );
  }

  #[test]
  fn base_only() {
    let mut lex = super::DateExpressionToken::lexer("now");
    expect!(super::expression(&mut lex, "now")).to(be_ok().value(ParsedDateExpression {
      base: DateBase::NOW,
      adjustments: vec![]
    }));

    let mut lex = super::DateExpressionToken::lexer("  today   ");
    expect!(super::expression(&mut lex, "now")).to(be_ok().value(ParsedDateExpression {
      base: DateBase::TODAY,
      adjustments: vec![]
    }));

    let mut lex = super::DateExpressionToken::lexer("tomorrow");
    expect!(super::expression(&mut lex, "now")).to(be_ok().value(ParsedDateExpression {
      base: DateBase::TOMORROW,
      adjustments: vec![]
    }));

    let mut lex = super::DateExpressionToken::lexer("yesterday");
    expect!(super::expression(&mut lex, "now")).to(be_ok().value(ParsedDateExpression {
      base: DateBase::YESTERDAY,
      adjustments: vec![]
    }));
  }

  #[test]
  fn op_and_duration() {
    let mut lex = super::DateExpressionToken::lexer("+1 day");
    expect!(super::expression(&mut lex, "+1 day")).to(be_ok().value(ParsedDateExpression {
      base: DateBase::NOW,
      adjustments: vec![
        Adjustment {
          adjustment_type: DateOffsetType::DAY,
          value: 1,
          operation: Operation::PLUS
        }
      ]
    }));

    let mut lex = super::DateExpressionToken::lexer("+ 2 weeks - 1 day");
    expect!(super::expression(&mut lex, "+ 2 weeks - 1 day")).to(be_ok().value(ParsedDateExpression {
      base: DateBase::NOW,
      adjustments: vec![
        Adjustment {
          adjustment_type: DateOffsetType::WEEK,
          value: 2,
          operation: Operation::PLUS
        },
        Adjustment {
          adjustment_type: DateOffsetType::DAY,
          value: 1,
          operation: Operation::MINUS
        }
      ]
    }));
  }

  #[test]
  fn base_and_op_and_duration() {
    let mut lex = super::DateExpressionToken::lexer("today + 2 week");
    expect!(super::expression(&mut lex, "today + 2 week")).to(be_ok().value(ParsedDateExpression {
      base: DateBase::TODAY,
      adjustments: vec![
        Adjustment {
          adjustment_type: DateOffsetType::WEEK,
          value: 2,
          operation: Operation::PLUS
        }
      ]
    }));

    let mut lex = super::DateExpressionToken::lexer("today 2 week");
    let result = super::expression(&mut lex, "today 2 week");
    assert_eq!(
      "|Error: Expected + or -
       |   ╭─[ expression:1:7 ]
       |   │
       | 1 │ today 2 week
       |   │       ┬ \u{0020}
       |   │       ╰── Expected + or - here
       |───╯
       |
       ".trim_margin_with("|").unwrap(),
      result.unwrap_err().to_string()
    );
  }
}
