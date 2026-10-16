//! Date/Time expressions to use with Date/Time generators.
//!
//! These expressions work relative to a base date + time, normally the current system clock. They
//! provide a reliable way to work with relative dates in tests.
//!
//! Given the base date-time of 2000-01-01T10:00Z, then the following will resolve to
//!
//! | Expression | Resulting date-time |
//! |------------|---------------------|
//! | ''                                 | '2000-01-01T10:00Z' |
//! | 'now'                              | '2000-01-01T10:00Z' |
//! | 'today'                            | '2000-01-01T10:00Z' |
//! | 'yesterday'                        | '1999-12-31T10:00Z' |
//! | 'tomorrow'                         | '2000-01-02T10:00Z' |
//! | '+ 1 day'                          | '2000-01-02T10:00Z' |
//! | '+ 1 week'                         | '2000-01-08T10:00Z' |
//! | '- 2 weeks'                        | '1999-12-18T10:00Z' |
//! | '+ 4 years'                        | '2004-01-01T10:00Z' |
//! | 'tomorrow+ 4 years'                | '2004-01-02T10:00Z' |
//! | 'next week'                        | '2000-01-08T10:00Z' |
//! | 'last month'                       | '1999-12-01T10:00Z' |
//! | 'next fortnight'                   | '2000-01-15T10:00Z' |
//! | 'next monday'                      | '2000-01-03T10:00Z' |
//! | 'last wednesday'                   | '1999-12-29T10:00Z' |
//! | 'next mon'                         | '2000-01-03T10:00Z' |
//! | 'last december'                    | '1999-12-01T10:00Z' |
//! | 'next jan'                         | '2001-01-01T10:00Z' |
//! | 'next june + 2 weeks'              | '2000-06-15T10:00Z' |
//! | 'last mon + 2 weeks'               | '2000-01-10T10:00Z' |
//! | '+ 1 day - 2 weeks'                | '1999-12-19T10:00Z' |
//! | 'last december + 2 weeks + 4 days' | '1999-12-19T10:00Z' |
//! | '@ now'                       | '2000-01-01T10:00Z' |
//! | '@ midnight'                  | '2000-01-01T00:00Z' |
//! | '@ noon'                      | '2000-01-01T12:00Z' |
//! | '@ 2 o\'clock'                | '2000-01-01T14:00Z' |
//! | '@ 12 o\'clock am'            | '2000-01-01T12:00Z' |
//! | '@ 1 o\'clock pm'             | '2000-01-01T13:00Z' |
//! | '@ + 1 hour'                  | '2000-01-01T11:00Z' |
//! | '@ - 2 minutes'               | '2000-01-01T09:58Z' |
//! | '@ + 4 seconds'               | '2000-01-01T10:00:04Z' |
//! | '@ + 4 milliseconds'          | '2000-01-01T10:00:00.004Z' |
//! | '@ midnight+ 4 minutes'       | '2000-01-01T00:04Z' |
//! | '@ next hour'                 | '2000-01-01T11:00Z' |
//! | '@ last minute'               | '2000-01-01T09:59Z' |
//! | '@ now + 2 hours - 4 minutes' | '2000-01-01T11:56Z' |
//! | '@  + 2 hours - 4 minutes'    | '2000-01-01T11:56Z' |
//! | 'today @ 1 o\'clock'                               | '2000-01-01T13:00Z' |
//! | 'yesterday @ midnight'                             | '1999-12-31T00:00Z' |
//! | 'yesterday @ midnight - 1 hour'                    | '1999-12-30T23:00Z' |
//! | 'tomorrow @ now'                                   | '2000-01-02T10:00Z' |
//! | '+ 1 day @ noon'                                   | '2000-01-02T12:00Z' |
//! | '+ 1 week @ +1 hour'                               | '2000-01-08T11:00Z' |
//! | '- 2 weeks @ now + 1 hour'                         | '1999-12-18T11:00Z' |
//! | '+ 4 years @ midnight'                             | '2004-01-01T00:00Z' |
//! | 'tomorrow+ 4 years @ 3 o\'clock + 40 milliseconds' | '2004-01-02T15:00:00.040Z' |
//! | 'next week @ next hour'                            | '2000-01-08T11:00Z' |
//! | 'last month @ last hour'                           | '1999-12-01T09:00Z' |

use std::ops::{Add, Sub};
use std::str::from_utf8;

use anyhow::anyhow;
use ariadne::{Config, Label, Report, ReportKind, Source};
use bytes::{BytesMut, BufMut};
use chrono::Duration;
use chrono::prelude::*;
use logos::{Logos, Span};

use crate::generators::date_expression_parser::{DateExpressionToken, ParsedDateExpression};
use crate::generators::time_expression_parser::{ParsedTimeExpression, TimeExpressionToken};

/// Enum representing the base for the date
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DateBase {
  NOW, TODAY, YESTERDAY, TOMORROW
}

/// Enum representing the base for the time
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TimeBase {
  Now, Midnight, Noon,
  Am {  hour: u8 },
  Pm {  hour: u8 },
  Next { hour: u8 }
}

impl TimeBase {
  pub(crate) fn of(hour: u64, ch: ClockHour, exp: &str, span: Span) -> anyhow::Result<TimeBase> {
    match ch {
      ClockHour::AM => match hour {
        1..=12 => Ok(TimeBase::Am { hour: hour as u8 }),
        _ => Err(error(exp, "hour 1 to 12", Some(span)))
      }
      ClockHour::PM => match hour {
        1..=12 => Ok(TimeBase::Pm { hour: hour as u8 }),
        _ => Err(error(exp, "hour 1 to 12", Some(span)))
      }
      ClockHour::NEXT => match hour {
        1..=12 => Ok(TimeBase::Next { hour: hour as u8 }),
        _ => Err(error(exp, "hour 1 to 12", Some(span)))
      }
    }
  }
}

/// Operation to apply to the base date
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Operation {
  PLUS, MINUS
}

/// Offset type for dates
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DateOffsetType {
  DAY, WEEK, MONTH, YEAR, MONDAY, TUESDAY, WEDNESDAY, THURSDAY, FRIDAY,
  SATURDAY, SUNDAY, JAN, FEB, MAR, APR, MAY, JUNE, JULY, AUG, SEP, OCT, NOV, DEC
}

/// Offset types for times
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TimeOffsetType {
  HOUR, MINUTE, SECOND, MILLISECOND
}

/// AM, PM or the next available hour
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClockHour {
  AM, PM, NEXT
}

/// Struct to represent an adjustment to a base date-time
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Adjustment<T> {
  /// The type of adjustmnent to make
  pub adjustment_type: T,
  /// The amount of the adjustment to make
  pub value: u64,
  /// If the adjustment is added or subtracted
  pub operation: Operation
}

fn parse_date_expression(expression: &str) -> anyhow::Result<ParsedDateExpression> {
  let mut lex = DateExpressionToken::lexer(expression);
  crate::generators::date_expression_parser::expression(&mut lex, expression)
}

fn parse_time_expression(expression: &str) -> anyhow::Result<ParsedTimeExpression> {
  let mut lex = TimeExpressionToken::lexer(expression);
  crate::generators::time_expression_parser::expression(&mut lex, expression)
}

/// Parse the date part of an expression. This will parse the expression, and then apply the
/// adjustments to the provided date to get a new date
pub fn execute_date_expression<Tz: TimeZone>(dt: &DateTime<Tz>, expression: &str) -> anyhow::Result<DateTime<Tz>> {
  if expression.is_empty() {
    Ok(dt.clone())
  } else {
    let result =  parse_date_expression(expression)?;
    let mut date = base_date(&result, dt);
    for adjustment in &result.adjustments {
      date = match adjustment.operation {
        Operation::PLUS => forward_date_by(adjustment, &date)?,
        Operation::MINUS => reverse_date_by(adjustment, &date)?
      }
    }
    Ok(date)
  }
}

/// Parse the time part of an expression
pub fn execute_time_expression<Tz: TimeZone>(dt: &DateTime<Tz>, expression: &str) -> anyhow::Result<DateTime<Tz>> {
  if expression.is_empty() {
    Ok(dt.clone())
  } else {
    let result = parse_time_expression(expression)?;
    let time = dt.with_minute(0)
      .map(|t| t.with_second(0))
      .flatten()
      .map(|t| t.with_nanosecond(0))
      .flatten()
      .unwrap_or(dt.clone());
    let base_time = match result.base {
      TimeBase::Now => dt.clone(),
      TimeBase::Midnight => time.with_hour(0).unwrap_or(dt.clone()),
      TimeBase::Noon => time.with_hour(12).unwrap_or(dt.clone()),
      TimeBase::Am { hour } => time.with_hour(hour as u32).unwrap_or(dt.clone()),
      TimeBase::Pm { hour } => time.with_hour((12 + hour) as u32).unwrap_or(dt.clone()),
      TimeBase::Next { hour } => if dt.hour() < 12 {
        time.with_hour((12 + hour) as u32).unwrap_or(dt.clone())
      } else {
        time.with_hour(hour as u32).unwrap_or(dt.clone())
      }
    };

    let mut date_time = base_time.clone();
    for adjustment in &result.adjustments {
      date_time = match adjustment.operation {
        Operation::PLUS => match adjustment.adjustment_type {
          TimeOffsetType::HOUR => date_time.add(Duration::try_hours(adjustment.value as i64)
            .ok_or_else(|| anyhow!("Adjustment value {} overflows the bounds", adjustment.value))?),
          TimeOffsetType::MINUTE => date_time.add(Duration::try_minutes(adjustment.value as i64)
            .ok_or_else(|| anyhow!("Adjustment value {} overflows the bounds", adjustment.value))?),
          TimeOffsetType::SECOND => date_time.add(Duration::try_seconds(adjustment.value as i64)
            .ok_or_else(|| anyhow!("Adjustment value {} overflows the bounds", adjustment.value))?),
          TimeOffsetType::MILLISECOND => date_time.add(Duration::try_milliseconds(adjustment.value as i64)
            .ok_or_else(|| anyhow!("Adjustment value {} overflows the bounds", adjustment.value))?)
        },
        Operation::MINUS => match adjustment.adjustment_type {
          TimeOffsetType::HOUR => date_time.sub(Duration::try_hours(adjustment.value as i64)
            .ok_or_else(|| anyhow!("Adjustment value {} overflows the bounds", adjustment.value))?),
          TimeOffsetType::MINUTE => date_time.sub(Duration::try_minutes(adjustment.value as i64)
            .ok_or_else(|| anyhow!("Adjustment value {} overflows the bounds", adjustment.value))?),
          TimeOffsetType::SECOND => date_time.sub(Duration::try_seconds(adjustment.value as i64)
            .ok_or_else(|| anyhow!("Adjustment value {} overflows the bounds", adjustment.value))?),
          TimeOffsetType::MILLISECOND => date_time.sub(Duration::try_milliseconds(adjustment.value as i64)
            .ok_or_else(|| anyhow!("Adjustment value {} overflows the bounds", adjustment.value))?)
        }
      }
    }

    Ok(date_time)
  }
}

/// Parse a date-time expression, given a base date-time
pub fn execute_datetime_expression<Tz: TimeZone>(dt: &DateTime<Tz>, expression: &str) -> anyhow::Result<DateTime<Tz>> {
  if !expression.is_empty() {
    if let Some((date_part, time_part)) = expression.split_once("@") {
      let date_result = execute_date_expression(dt, date_part);
      let time_result = match &date_result {
        Ok(result) => execute_time_expression(result, time_part),
        Err(_) => execute_time_expression(dt, time_part)
      };
      match (date_result, time_result) {
        (Err(err1), Err(err2)) => Err(anyhow!("{err1}\n{err2}")),
        (Err(err), Ok(_)) => Err(err),
        (Ok(_), Err(err)) => Err(err),
        (Ok(_), Ok(t)) => Ok(t)
      }
    } else {
      execute_date_expression(dt, expression)
    }
  } else {
    Ok(dt.clone())
  }
}

fn forward_date_by<Tz: TimeZone>(adjustment: &Adjustment<DateOffsetType>, date: &DateTime<Tz>) -> anyhow::Result<DateTime<Tz>> {
  Ok(match adjustment.adjustment_type {
    DateOffsetType::DAY => date.clone().add(Duration::try_days(adjustment.value as i64)
      .ok_or_else(|| anyhow!("Adjustment value {} overflows the bounds", adjustment.value))?),
    DateOffsetType::WEEK => date.clone().add(Duration::try_weeks(adjustment.value as i64)
      .ok_or_else(|| anyhow!("Adjustment value {} overflows the bounds", adjustment.value))?),
    DateOffsetType::MONTH => roll_month(date, adjustment.value as i64),
    DateOffsetType::YEAR => {
      let date = date.clone();
      let year = date.year();
      date.with_year(year + adjustment.value as i32).unwrap_or(date)
    },
    DateOffsetType::MONDAY => adjust_date_up_to(date, |d| d.weekday() == Weekday::Mon),
    DateOffsetType::TUESDAY => adjust_date_up_to(date, |d| d.weekday() == Weekday::Tue),
    DateOffsetType::WEDNESDAY => adjust_date_up_to(date, |d| d.weekday() == Weekday::Wed),
    DateOffsetType::THURSDAY => adjust_date_up_to(date, |d| d.weekday() == Weekday::Thu),
    DateOffsetType::FRIDAY => adjust_date_up_to(date, |d| d.weekday() == Weekday::Fri),
    DateOffsetType::SATURDAY => adjust_date_up_to(date, |d| d.weekday() == Weekday::Sat),
    DateOffsetType::SUNDAY => adjust_date_up_to(date, |d| d.weekday() == Weekday::Sun),
    DateOffsetType::JAN => adjust_date_up_to(date, |d| d.month() == 1),
    DateOffsetType::FEB => adjust_date_up_to(date, |d| d.month() == 2),
    DateOffsetType::MAR => adjust_date_up_to(date, |d| d.month() == 3),
    DateOffsetType::APR => adjust_date_up_to(date, |d| d.month() == 4),
    DateOffsetType::MAY => adjust_date_up_to(date, |d| d.month() == 5),
    DateOffsetType::JUNE => adjust_date_up_to(date, |d| d.month() == 6),
    DateOffsetType::JULY => adjust_date_up_to(date, |d| d.month() == 7),
    DateOffsetType::AUG => adjust_date_up_to(date, |d| d.month() == 8),
    DateOffsetType::SEP => adjust_date_up_to(date, |d| d.month() == 9),
    DateOffsetType::OCT => adjust_date_up_to(date, |d| d.month() == 10),
    DateOffsetType::NOV => adjust_date_up_to(date, |d| d.month() == 11),
    DateOffsetType::DEC => adjust_date_up_to(date, |d| d.month() == 12)
  })
}

/// Rolls the date forward one day at a time until the predicate is true
fn adjust_date_up_to<Tz: TimeZone>(
  date: &DateTime<Tz>,
  predicate: fn(&DateTime<Tz>) -> bool
) -> DateTime<Tz> {
  let mut date = date.clone();
  let one_day_duration = Duration::try_days(1).unwrap(); // OK to unwrap, 1 will never overflow

  while predicate(&date) {
    date = date.add(one_day_duration);
  }

  while !predicate(&date) {
    date = date.add(one_day_duration);
  }

  date
}

/// Rolls the date backwards one day at a time until the predicate is true
fn adjust_date_down_to<Tz: TimeZone>(
  date: &DateTime<Tz>,
  predicate: fn(&DateTime<Tz>) -> bool
) -> DateTime<Tz> {
  let mut date = date.clone();
  let one_day_duration = Duration::try_days(1).unwrap(); // OK to unwrap, 1 will never overflow;

  while predicate(&date) {
    date = date.sub(one_day_duration);
  }

  while !predicate(&date) {
    date = date.sub(one_day_duration);
  }

  date
}

/// Rolls the month by the adjustment one day at a time
fn roll_month<Tz: TimeZone>(date: &DateTime<Tz>, months: i64) -> DateTime<Tz> {
  let mut date = date.clone();
  let day = date.day();
  let one_day_duration = Duration::try_days(1).unwrap(); // OK to unwrap, 1 will never overflow
  let mut month_count = 0;

  if months > 0 {
    let mut month = date.month();
    while month_count < months {
      date = date.add(one_day_duration);
      if date.month() != month {
        month = date.month();
        month_count += 1;
      }
    }
    date.with_day(day).unwrap_or(date)
  } else if months < 0 {
    let mut month = date.month();
    while month_count > months {
      date = date.sub(one_day_duration);
      if date.month() != month {
        month = date.month();
        month_count -= 1;
      }
    }
    date.with_day(day).unwrap_or(date)
  } else {
    date
  }
}

fn reverse_date_by<Tz: TimeZone>(adjustment: &Adjustment<DateOffsetType>, date: &DateTime<Tz>) -> anyhow::Result<DateTime<Tz>> {
  Ok(match adjustment.adjustment_type {
    DateOffsetType::DAY => date.clone().sub(Duration::try_days(adjustment.value as i64)
      .ok_or_else(|| anyhow!("Adjustment value {} overflows the bounds", adjustment.value))?),
    DateOffsetType::WEEK => date.clone().sub(Duration::try_weeks(adjustment.value as i64)
      .ok_or_else(|| anyhow!("Adjustment value {} overflows the bounds", adjustment.value))?),
    DateOffsetType::MONTH => roll_month(date, -(adjustment.value as i64)),
    DateOffsetType::YEAR => {
      let date = date.clone();
      let year = date.year();
      date.with_year(year - adjustment.value as i32).unwrap_or(date)
    },
    DateOffsetType::MONDAY => adjust_date_down_to(date, |d| d.weekday() == Weekday::Mon),
    DateOffsetType::TUESDAY => adjust_date_down_to(date, |d| d.weekday() == Weekday::Tue),
    DateOffsetType::WEDNESDAY => adjust_date_down_to(date, |d| d.weekday() == Weekday::Wed),
    DateOffsetType::THURSDAY => adjust_date_down_to(date, |d| d.weekday() == Weekday::Thu),
    DateOffsetType::FRIDAY => adjust_date_down_to(date, |d| d.weekday() == Weekday::Fri),
    DateOffsetType::SATURDAY => adjust_date_down_to(date, |d| d.weekday() == Weekday::Sat),
    DateOffsetType::SUNDAY => adjust_date_down_to(date, |d| d.weekday() == Weekday::Sun),
    DateOffsetType::JAN => adjust_date_down_to(date, |d| d.month() == 1).with_day(1).unwrap_or_else(|| date.clone()),
    DateOffsetType::FEB => adjust_date_down_to(date, |d| d.month() == 2).with_day(1).unwrap_or_else(|| date.clone()),
    DateOffsetType::MAR => adjust_date_down_to(date, |d| d.month() == 3).with_day(1).unwrap_or_else(|| date.clone()),
    DateOffsetType::APR => adjust_date_down_to(date, |d| d.month() == 4).with_day(1).unwrap_or_else(|| date.clone()),
    DateOffsetType::MAY => adjust_date_down_to(date, |d| d.month() == 5).with_day(1).unwrap_or_else(|| date.clone()),
    DateOffsetType::JUNE => adjust_date_down_to(date, |d| d.month() == 6).with_day(1).unwrap_or_else(|| date.clone()),
    DateOffsetType::JULY => adjust_date_down_to(date, |d| d.month() == 7).with_day(1).unwrap_or_else(|| date.clone()),
    DateOffsetType::AUG => adjust_date_down_to(date, |d| d.month() == 8).with_day(1).unwrap_or_else(|| date.clone()),
    DateOffsetType::SEP => adjust_date_down_to(date, |d| d.month() == 9).with_day(1).unwrap_or_else(|| date.clone()),
    DateOffsetType::OCT => adjust_date_down_to(date, |d| d.month() == 10).with_day(1).unwrap_or_else(|| date.clone()),
    DateOffsetType::NOV => adjust_date_down_to(date, |d| d.month() == 11).with_day(1).unwrap_or_else(|| date.clone()),
    DateOffsetType::DEC => adjust_date_down_to(date, |d| d.month() == 12).with_day(1).unwrap_or_else(|| date.clone())
  })
}

fn base_date<Tz: TimeZone>(result: &ParsedDateExpression, base: &DateTime<Tz>) -> DateTime<Tz> {
  match result.base {
    DateBase::NOW | DateBase::TODAY => base.clone(),
    // Ok to unwrap, 1 will never overflow bounds
    DateBase::YESTERDAY => base.clone().sub(Duration::try_days(1).unwrap()),
    DateBase::TOMORROW => base.clone().add(Duration::try_days(1).unwrap())
  }
}

pub(crate) fn error(exp: &str, expected: &str, span: Option<Span>) -> anyhow::Error {
  let mut buffer = BytesMut::new().writer();
  let span = match span {
    None => {
      let i = exp.len();
      i..i
    }
    Some(span) => span
  };
  let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
    .with_config(Config::default().with_color(false))
    .with_message(format!("Expected {}", expected))
    .with_label(Label::new(("expression", span)).with_message(format!("Expected {} here", expected)))
    .finish();
  report.write(("expression", Source::from(exp)), &mut buffer).unwrap();
  let message = from_utf8(&*buffer.get_ref()).unwrap().to_string();
  anyhow!(message)
}

#[cfg(test)]
mod tests {
  use chrono::prelude::*;
  use expectest::prelude::*;
  use rstest::rstest;

  use super::*;

  #[rstest]
  //     expression,            expected
  #[case("",                    "2000-01-01 10:00:00 UTC")]
  #[case("now",                 "2000-01-01 10:00:00 UTC")]
  #[case("today",               "2000-01-01 10:00:00 UTC")]
  #[case("yesterday",           "1999-12-31 10:00:00 UTC")]
  #[case("tomorrow",            "2000-01-02 10:00:00 UTC")]
  #[case("+ 1 day",             "2000-01-02 10:00:00 UTC")]
  #[case("+ 1 week",            "2000-01-08 10:00:00 UTC")]
  #[case("- 2 weeks",           "1999-12-18 10:00:00 UTC")]
  #[case("+ 4 years",           "2004-01-01 10:00:00 UTC")]
  #[case("tomorrow+ 4 years",   "2004-01-02 10:00:00 UTC")]
  #[case("next week",           "2000-01-08 10:00:00 UTC")]
  #[case("last month",          "1999-12-01 10:00:00 UTC")]
  #[case("next fortnight",      "2000-01-15 10:00:00 UTC")]
  #[case("next monday",         "2000-01-03 10:00:00 UTC")]
  #[case("last wednesday",      "1999-12-29 10:00:00 UTC")]
  #[case("next mon",            "2000-01-03 10:00:00 UTC")]
  #[case("last december",       "1999-12-01 10:00:00 UTC")]
  #[case("next jan",            "2001-01-01 10:00:00 UTC")]
  #[case("next june + 2 weeks", "2000-06-15 10:00:00 UTC")]
  #[case("last mon + 2 weeks",  "2000-01-10 10:00:00 UTC")]
  fn date_expressions(#[case] expression: &str, #[case] expected: &str) {
    let dt = Utc.with_ymd_and_hms(2000, 1, 1, 10, 0, 0).unwrap();
    expect!(execute_date_expression(&dt, expression).unwrap().to_string()).to(be_equal_to(expected));
  }

  #[rstest]
  //     expression,                  expected
  #[case("",                          "10:00:00")]
  #[case("now",                       "10:00:00")]
  #[case("midnight",                  "00:00:00")]
  #[case("noon",                      "12:00:00")]
  #[case("1 o'clock",                 "13:00:00")]
  #[case("1 o'clock am",              "01:00:00")]
  #[case("1 o'clock pm",              "13:00:00")]
  #[case("+ 1 hour",                  "11:00:00")]
  #[case("- 2 minutes",               "09:58:00")]
  #[case("+ 4 seconds",               "10:00:04")]
  #[case("+ 4 milliseconds",          "10:00:00.004")]
  #[case("midnight+ 4 minutes",       "00:04:00")]
  #[case("next hour",                 "11:00:00")]
  #[case("last minute",               "09:59:00")]
  #[case("now + 2 hours - 4 minutes", "11:56:00")]
  #[case(" + 2 hours - 4 minutes",    "11:56:00")]
  fn time_expressions(#[case] expression: &str, #[case] expected: &str) {
    let dt = Utc.with_ymd_and_hms(2000, 1, 1, 10, 0, 0).unwrap();
    expect!(execute_time_expression(&dt, expression).unwrap().time().to_string()).to(be_equal_to(expected));
  }

  #[rstest]
  //     expression,                                         expected
  #[case("today @ 1 o\'clock",                               "2000-01-01T13:00:00+00:00")]
  #[case("yesterday @ midnight",                             "1999-12-31T00:00:00+00:00")]
  #[case("yesterday @ midnight - 1 hour",                    "1999-12-30T23:00:00+00:00")]
  #[case("tomorrow @ now",                                   "2000-01-02T10:00:00+00:00")]
  #[case("+ 1 day @ noon",                                   "2000-01-02T12:00:00+00:00")]
  #[case("+ 1 week @ +1 hour",                               "2000-01-08T11:00:00+00:00")]
  #[case("- 2 weeks @ now + 1 hour",                         "1999-12-18T11:00:00+00:00")]
  #[case("+ 4 years @ midnight",                             "2004-01-01T00:00:00+00:00")]
  #[case("tomorrow+ 4 years @ 3 o\'clock + 40 milliseconds", "2004-01-02T15:00:00.040+00:00")]
  #[case("next week @ next hour",                            "2000-01-08T11:00:00+00:00")]
  #[case("last month @ last hour",                           "1999-12-01T09:00:00+00:00")]
  fn datetime_expressions(#[case] expression: &str, #[case] expected: &str) {
    let dt = Utc.with_ymd_and_hms(2000, 1, 1, 10, 0, 0).unwrap();
    expect!(execute_datetime_expression(&dt, expression).unwrap().to_rfc3339()).to(be_equal_to(expected));
  }

  #[test]
  fn base_date_test() {
    let dt = Utc.with_ymd_and_hms(2000, 1, 1, 10, 0, 0).unwrap();

    expect!(base_date(&ParsedDateExpression { base: DateBase::NOW, adjustments: vec![] }, &dt))
      .to(be_equal_to(Utc.with_ymd_and_hms(2000, 1, 1, 10, 0, 0).unwrap()));
    expect!(base_date(&ParsedDateExpression { base: DateBase::TODAY, adjustments: vec![] }, &dt))
      .to(be_equal_to(Utc.with_ymd_and_hms(2000, 1, 1, 10, 0, 0).unwrap()));
    expect!(base_date(&ParsedDateExpression { base: DateBase::TOMORROW, adjustments: vec![] }, &dt))
      .to(be_equal_to(Utc.with_ymd_and_hms(2000, 1, 2, 10, 0, 0).unwrap()));
    expect!(base_date(&ParsedDateExpression { base: DateBase::YESTERDAY, adjustments: vec![] }, &dt))
      .to(be_equal_to(Utc.with_ymd_and_hms(1999, 12, 31, 10, 0, 0).unwrap()));
  }

  #[test]
  fn forward_date_by_test() {
    let dt = Utc.with_ymd_and_hms(2020, 1, 1, 10, 0, 0).unwrap();
    let dt2 = Utc.with_ymd_and_hms(2020, 12, 1, 10, 0, 0).unwrap();

    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::DAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 1, 2, 10, 0, 0).unwrap()));

    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::MONTH, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 2, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::MONTH, value: 4, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 5, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::MONTH, value: 13, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2021, 2, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::MONTH, value: 1, operation: Operation::PLUS }, &dt2).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap()));

    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::YEAR, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::WEEK, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 1, 8, 10, 0, 0).unwrap()));

    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::MONDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 1, 6, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::TUESDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 1, 7, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::WEDNESDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 1, 8, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::THURSDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 1, 2, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::FRIDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 1, 3, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::SATURDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 1, 4, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::SUNDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 1, 5, 10, 0, 0).unwrap()));

    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::JAN, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::FEB, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 2, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::MAR, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 3, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::APR, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 4, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::MAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 5, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::JUNE, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 6, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::JULY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 7, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::AUG, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 8, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::SEP, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 9, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::OCT, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 10, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::NOV, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 11, 1, 10, 0, 0).unwrap()));
    expect!(forward_date_by(&Adjustment { adjustment_type: DateOffsetType::DEC, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2020, 12, 1, 10, 0, 0).unwrap()));
  }

  #[test]
  fn reverse_date_by_test() {
    let dt = Utc.with_ymd_and_hms(2020, 1, 1, 10, 0, 0).unwrap();

    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::DAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 12, 31, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::MONTH, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 12, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::YEAR, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 1, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::WEEK, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 12, 25, 10, 0, 0).unwrap()));

    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::MONDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 12, 30, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::TUESDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 12, 31, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::WEDNESDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 12, 25, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::THURSDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 12, 26, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::FRIDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 12, 27, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::SATURDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 12, 28, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::SUNDAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 12, 29, 10, 0, 0).unwrap()));

    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::JAN, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 1, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::FEB, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 2, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::MAR, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 3, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::APR, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 4, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::MAY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 5, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::JUNE, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 6, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::JULY, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 7, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::AUG, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 8, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::SEP, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 9, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::OCT, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 10, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::NOV, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 11, 1, 10, 0, 0).unwrap()));
    expect!(reverse_date_by(&Adjustment { adjustment_type: DateOffsetType::DEC, value: 1, operation: Operation::PLUS }, &dt).unwrap())
      .to(be_equal_to(Utc.with_ymd_and_hms(2019, 12, 1, 10, 0, 0).unwrap()));
  }

  #[test]
  fn role_month_test() {
    let dt = Utc.with_ymd_and_hms(2000, 4, 13, 10, 0, 0).unwrap();

    expect!(roll_month(&dt, 0))
      .to(be_equal_to(Utc.with_ymd_and_hms(2000, 4, 13, 10, 0, 0).unwrap()));
    expect!(roll_month(&dt, 1))
      .to(be_equal_to(Utc.with_ymd_and_hms(2000, 5, 13, 10, 0, 0).unwrap()));
    expect!(roll_month(&dt, -1))
      .to(be_equal_to(Utc.with_ymd_and_hms(2000, 3, 13, 10, 0, 0).unwrap()));
    expect!(roll_month(&dt, 10))
      .to(be_equal_to(Utc.with_ymd_and_hms(2001, 2, 13, 10, 0, 0).unwrap()));
    expect!(roll_month(&dt, -10))
      .to(be_equal_to(Utc.with_ymd_and_hms(1999, 6, 13, 10, 0, 0).unwrap()));
  }
}
