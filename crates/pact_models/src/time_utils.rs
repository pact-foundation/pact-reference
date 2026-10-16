//! Utilities for dealing with time and date values. These are based on the Java DateTimeFormatter
//!
//!   |Symbol| Meaning                   | Presentation |  Examples |
//!   |------| -------                   | ------------ |  ------- |
//!   |G     | era                       | text         |  AD; Anno Domini; A |
//!   |u     | year                      | year         |  2004; 04 |
//!   |y     | year-of-era               | year         |  2004; 04 |
//!   |D     | day-of-year               | number       |  189 |
//!   |M/L   | month-of-year             | number/text  |  7; 07; Jul; July; J |
//!   |d     | day-of-month              | number       |  10 |
//!   |Q/q   | quarter-of-year           | number/text  |  3; 03; Q3; 3rd quarter |
//!   |Y     | week-based-year           | year         |  1996; 96 |
//!   |w     | week-of-week-based-year   | number       |  27 |
//!   |W     | week-of-month             | number       |  4 |
//!   |E     | day-of-week               | text         |  Tue; Tuesday; T |
//!   |e/c   | localized day-of-week     | number/text  |  2; 02; Tue; Tuesday; T |
//!   |F     | week-of-month             | number       |  3 |
//!   |a     | am-pm-of-day              | text         |  PM |
//!   |h     | clock-hour-of-am-pm (1-12)| number       |  12 |
//!   |K     | hour-of-am-pm (0-11)      | number       |  0 |
//!   |k     | clock-hour-of-am-pm (1-24)| number       |  0 |
//!   |H     | hour-of-day (0-23)        | number       |  0 |
//!   |m     | minute-of-hour            | number       |  30 |
//!   |s     | second-of-minute          | number       |  55 |
//!   |S     | fraction-of-second        | fraction     |  978 |
//!   |A     | milli-of-day              | number       |  1234 |
//!   |n     | nano-of-second            | number       |  987654321 |
//!   |N     | nano-of-day               | number       |  1234000000 |
//!   |V     | time-zone ID              | zone-id      |  America/Los_Angeles; Z; -08:30 |
//!   |z     | time-zone name            | zone-name    |  Pacific Standard Time; PST |
//!   |O     | localized zone-offset     | offset-O     |  GMT+8; GMT+08:00; UTC-08:00; |
//!   |X     | zone-offset 'Z' for zero  | offset-X     |  Z; -08; -0830; -08:30; -083015; -08:30:15; |
//!   |x     | zone-offset               | offset-x     |  +0000; -08; -0830; -08:30; -083015; -08:30:15; |
//!   |Z     | zone-offset               | offset-Z     |  +0000; -0800; -08:00; |
//!   |'     | escape for text           | delimiter    | |
//!   |''    | single quote              | literal      |  ' |

use std::fmt::{Display, Formatter};

use anyhow::anyhow;
use chrono::Local;
use gregorian::Month;
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag, tag_no_case, take_while_m_n};
use nom::character::complete::{alphanumeric1, char, digit1};
use nom::combinator::{opt, value};
use nom::Err::{Error, Failure};
use nom::error::{ErrorKind, ParseError};
use nom::IResult;
use nom::multi::many1;
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use tracing::{debug, error, trace, warn};

use crate::timezone_db::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
/// Tokens for DateTime patterns
pub enum DateTimePatternToken {
  Era(usize),
  Year(usize),
  Month(usize),
  MonthNum(usize),
  Text(String),
  WeekInYear,
  WeekInMonth(bool),
  DayInYear,
  DayInMonth,
  DayName(usize),
  DayOfWeek(usize),
  AmPm,
  Hour24,
  Hour24ZeroBased,
  Hour12,
  Hour12ZeroBased,
  Minute,
  Second,
  // TODO: This should be fraction of a second
  Millisecond(usize),
  Nanosecond(usize),
  TimezoneOffset(usize),
  TimezoneOffsetX(usize),
  TimezoneOffsetXZZero(usize),
  TimezoneOffsetGmt(usize),
  TimezoneName(usize),
  TimezoneId(usize),
  QuarterOfYear(usize),
  QuarterOfYearNum(usize),
  MillisecondOfDay,
  NanosecondOfDay
}

#[derive(Debug, PartialEq, Clone)]
/// Errors when parsing a date time pattern
pub enum DateTimePatternError<I> {
  /// Too many pattern letters were encountered
  TooManyPatternLetters(String, usize),
  /// Unparsed characters remaining
  RemainingCharacters(String),
  /// Nom error occurred
  Nom(I, ErrorKind),
}

impl<I> ParseError<I> for DateTimePatternError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) -> Self {
    DateTimePatternError::Nom(input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

impl <I> Display for DateTimePatternError<I> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DateTimePatternError::TooManyPatternLetters(s, _count)  => f.write_str(s),
      DateTimePatternError::RemainingCharacters(s) => f.write_fmt(format_args!("Remaining unmatched characters at '{}'", s)),
      DateTimePatternError::Nom(_s, err) => f.write_str(err.description())
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
#[allow(missing_docs)]
/// Errors when parsing date time values
pub enum DateTimeError<I> {
  InvalidDayInYear(String),
  InvalidDayInMonth(String),
  InvalidMonth(String),
  InvalidQuarter(String),
  InvalidWeekInYear(String),
  InvalidDayOfWeek(String),
  InvalidHour(String),
  InvalidMinute(String),
  // TODO: This is actually for a fraction of a second
  #[deprecated(note = "Use second fraction (InvalidSecondFraction)")]
  InvalidMillisecond(String),
  FullTimezonesNotSupported(String),
  InvalidTimezone(String),
  InvalidSecondFraction(String),
  /// Nom error occurred
  Nom(I, ErrorKind),
}

impl<I> ParseError<I> for DateTimeError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) -> Self {
    DateTimeError::Nom(input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

fn is_digit(ch: char) -> bool {
  ch.is_ascii_digit()
}

fn is_uppercase(ch: char) -> bool {
  ch.is_ascii_uppercase()
}

fn validate_number(m: &str, num_type: String, lower: usize, upper: usize) -> Result<&str, String> {
  match m.parse::<usize>() {
    Ok(v) => if v >= lower && v <= upper {
      Ok(m)
    } else {
      Err(format!("Invalid {} {}", num_type, v))
    },
    Err(err) => Err(format!("{}", err))
  }
}

fn era_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("G")(s)
    .and_then(|(remaining, result)| {
      if result.len() > 5 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Era ('G'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::Era(result.len())))
      }
    })
}

fn ampm_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("a")(s)
    .and_then(|(remaining, result)| {
      if result.len() > 1 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for AM/PM ('a'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::AmPm))
      }
    })
}

fn week_in_year_month_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("w"), is_a("W"), is_a("F")))(s)
    .and_then(|(remaining, result)| {
      if result.len() > 2 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Week in Month ('W' or 'F'): {}", result.len()), result.len())))
      } else if result.starts_with('w') {
        Ok((remaining, DateTimePatternToken::WeekInYear))
      } else {
        Ok((remaining, DateTimePatternToken::WeekInMonth(result.starts_with('W'))))
      }
    })
}

fn day_in_year_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("D")(s).map(|(remaining, _)| {
    (remaining, DateTimePatternToken::DayInYear)
  })
}

fn day_in_month_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("d")(s)
    .and_then(|(remaining, result)| {
      if result.len() > 2 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Day in Month ('d'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::DayInMonth))
      }
    })
}

fn day_name_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("E")(s)
    .and_then(|(remaining, result)| {
      if result.len() > 5 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Day of Week ('E'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::DayName(result.len())))
      }
    })
}

fn day_of_week_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("e"), is_a("c")))(s)
    .and_then(|(remaining, result)| {
      if result.len() > 5 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Day of Week ('e'): {}", result.len()), result.len())))
      } else if result.starts_with('c') && result.len() > 1 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Day of Week ('c'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::DayOfWeek(result.len())))
      }
    })
}

fn year_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("y"), is_a("Y"), is_a("u")))(s).map(|(remaining, result)| {
    (remaining, DateTimePatternToken::Year(result.len()))
  })
}

fn month_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("M"), is_a("L")))(s).and_then(|(remaining, result)| {
    if result.len() > 5 {
      Err(Failure(DateTimePatternError::TooManyPatternLetters(
        format!("Too many pattern letters for Month ('M' or 'L'): {}", result.len()), result.len())))
    } else if result.starts_with('M') {
      Ok((remaining, DateTimePatternToken::Month(result.len())))
    } else {
      Ok((remaining, DateTimePatternToken::MonthNum(result.len())))
    }
  })
}

fn quarter_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("q"), is_a("Q")))(s).and_then(|(remaining, result)| {
    if result.len() > 5 {
      Err(Failure(DateTimePatternError::TooManyPatternLetters(
        format!("Too many pattern letters for Quarter ('q' or 'Q'): {}", result.len()), result.len())))
    } else if result.starts_with('Q') {
      Ok((remaining, DateTimePatternToken::QuarterOfYear(result.len())))
    } else if result.starts_with('q') && result.len() > 2 {
      Err(Failure(DateTimePatternError::TooManyPatternLetters(
        format!("Too many pattern letters for Quarter ('q'): {}", result.len()), result.len())))
    } else {
      Ok((remaining, DateTimePatternToken::QuarterOfYearNum(result.len())))
    }
  })
}

fn quoted_text_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  delimited(char('\''), many1(alt((tag("''"), is_not("'")))), char('\''))(s)
    .map(|(remaining, result)| {
      (remaining, DateTimePatternToken::Text(result.join("").chars().coalesce(|x, y| {
        if x == '\'' && y == '\'' { Ok('\'') } else { Err((x, y)) }
      }).collect()))
    })
}

fn quote_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  value(DateTimePatternToken::Text("'".into()), tag("''"))(s)
}

fn hour_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("h"), is_a("H"), is_a("k"), is_a("K")))(s).and_then(|(remaining, result)| {
    if result.len() > 2 {
      Err(Failure(DateTimePatternError::TooManyPatternLetters(
        format!("Too many pattern letters for Hour ('h', 'H', 'k' or 'K'): {}", result.len()), result.len())))
    } else if result.starts_with('h') {
      Ok((remaining, DateTimePatternToken::Hour12))
    } else if result.starts_with('H') {
      Ok((remaining, DateTimePatternToken::Hour24ZeroBased))
    } else if result.starts_with('k') {
      Ok((remaining, DateTimePatternToken::Hour24))
    } else {
      Ok((remaining, DateTimePatternToken::Hour12ZeroBased))
    }
  })
}

fn minute_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("m")(s)
    .and_then(|(remaining, result)| {
      if result.len() > 2 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Minute ('m'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::Minute))
      }
    })
}

fn second_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("s")(s)
    .and_then(|(remaining, result)| {
      if result.len() > 2 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Minute ('m'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::Second))
      }
    })
}

fn millisecond_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("S")(s)
    .map(|(remaining, result)| {
      (remaining, DateTimePatternToken::Millisecond(result.len()))
    })
}

fn nanosecond_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("n")(s)
      .map(|(remaining, result)| {
        (remaining, DateTimePatternToken::Nanosecond(result.len()))
      })
}

fn millisecond_of_day_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  value(DateTimePatternToken::MillisecondOfDay, is_a("A"))(s)
}

fn nanosecond_of_day_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  value(DateTimePatternToken::NanosecondOfDay, is_a("N"))(s)
}

fn timezone_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("x"), is_a("X"), is_a("Z"), is_a("O"), is_a("z"), is_a("V")))(s).and_then(|(remaining, result)| {
    if result.len() > 5 {
      Err(Failure(DateTimePatternError::TooManyPatternLetters(
        format!("Too many pattern letters for Timezone Offset ('x', 'X', 'O', 'z', or 'Z'): {}", result.len()), result.len())))
    } else if result.starts_with('Z') {
      Ok((remaining, DateTimePatternToken::TimezoneOffset(result.len())))
    } else if result.starts_with('x') {
      Ok((remaining, DateTimePatternToken::TimezoneOffsetX(result.len())))
    } else if result.starts_with('O') {
      if result.len() > 4 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Timezone Offset ('O'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::TimezoneOffsetGmt(result.len())))
      }
    } else if result.starts_with('V') {
      if result.len() > 2 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Timezone ID ('V'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::TimezoneId(result.len())))
      }
    } else if result.starts_with('z') {
      if result.len() > 4 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Timezone Offset ('z'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::TimezoneName(result.len())))
      }
    } else {
      Ok((remaining, DateTimePatternToken::TimezoneOffsetXZZero(result.len())))
    }
  })
}

fn text_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_not("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'{}[]")(s).map(|(remaining, result)| {
    (remaining, DateTimePatternToken::Text(result.to_string()))
  })
}

fn era(s: &str, _count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  alt((tag_no_case("ad"), tag_no_case("bc")))(s)
    .map(|(remaining, result)| (remaining, result.into()))
}

fn ampm(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  alt((tag_no_case("am"), tag_no_case("pm")))(s)
    .map(|(remaining, result)| (remaining, result.into()))
}

fn year(s: &str, count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, count, is_digit)(s)
    .map(|(remaining, result)| (remaining, result.into()))
}

fn month_num(s: &str, _count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "month".into(), 1, 12) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidMonth(result.to_string())))
    }
  })
}

fn month(s: &str, count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  if count <= 2 {
    month_num(s, count)
  } else if count == 3 {
    alt((
      tag_no_case("jan"),
      tag_no_case("feb"),
      tag_no_case("mar"),
      tag_no_case("apr"),
      tag_no_case("may"),
      tag_no_case("jun"),
      tag_no_case("jul"),
      tag_no_case("aug"),
      tag_no_case("sep"),
      tag_no_case("oct"),
      tag_no_case("nov"),
      tag_no_case("dec"),
    ))(s).map(|(remaining, result)| (remaining, result.into()))
  } else {
    alt((
      tag_no_case("january"),
      tag_no_case("february"),
      tag_no_case("march"),
      tag_no_case("april"),
      tag_no_case("may"),
      tag_no_case("june"),
      tag_no_case("july"),
      tag_no_case("august"),
      tag_no_case("september"),
      tag_no_case("october"),
      tag_no_case("november"),
      tag_no_case("december"),
    ))(s).map(|(remaining, result)| (remaining, result.into()))
  }
}

fn week_in_year(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "week in year".into(), 1, 56) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidWeekInYear(result.to_string())))
    }
  })
}

fn week_in_month(s: &str, from_one: bool) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    let lower = if from_one { 1 } else { 0 };
    let upper = if from_one { 5 } else { 4 };
    match validate_number(result, "week in month".into(), lower, upper) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidWeekInYear(result.to_string())))
    }
  })
}

fn day_in_year(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 3, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "day in year".into(), 1, 356) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidDayInYear(result.to_string())))
    }
  })
}

fn day_in_month(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "day in month".into(), 1, 31) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidDayInMonth(result.to_string())))
    }
  })
}

fn day_of_week(s: &str, count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  if count > 2 {
    day_of_week_name(s, count)
  } else {
    take_while_m_n(1, 1, is_digit)(s).and_then(|(remaining, result)|{
      match validate_number(result, "day of week".into(), 1, 7) {
        Ok(_) => Ok((remaining, result.into())),
        Err(_err) => Err(Error(DateTimeError::InvalidDayOfWeek(result.to_string())))
      }
    })
  }
}

fn hour_24(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "hour".into(), 1, 24) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidHour(result.to_string())))
    }
  })
}

fn hour_24_0(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "hour (zero-based)".into(), 0, 23) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidHour(result.to_string())))
    }
  })
}
fn hour_12(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "hour".into(), 1, 12) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidHour(result.to_string())))
    }
  })
}

fn hour_12_0(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "hour (zero-based)".into(), 0, 11) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidHour(result.to_string())))
    }
  })
}

fn minute(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "minute".into(), 0, 59) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidMinute(result.to_string())))
    }
  })
}

fn second(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "second".into(), 0, 60) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidMinute(result.to_string())))
    }
  })
}

fn second_fraction(s: &str, count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, count, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "fraction of a second".into(), 0, usize::pow(10, count as u32) - 1) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidSecondFraction(result.to_string())))
    }
  })
}

fn timezone_hour_min(s: &str) -> IResult<&str, &str, DateTimeError<&str>> {
  tuple((is_a("+-"), hour_12_0, tag(":"), minute))(s)
    .map(|(remaining, _result)| {
      (remaining, "")
    })
}

fn timezone_long_offset(s: &str, d: usize) -> IResult<&str, String, DateTimeError<&str>> {
  match d {
    1 => preceded(is_a("+-"), tuple((hour_12_0, opt(minute))))(s)
        .map(|(remaining, result)| {
          (remaining, result.0 + &result.1.unwrap_or_else(|| "".to_string()))
        }),
    2 => preceded(is_a("+-"), tuple((hour_12_0, minute)))(s)
        .map(|(remaining, result)| {
          (remaining, result.0 + &result.1)
        }),
    3 => preceded(is_a("+-"), tuple((hour_12_0, tag(":"), minute)))(s)
        .map(|(remaining, result)| {
          (remaining, result.0 + result.1 + &result.2)
        }),
    4 => preceded(is_a("+-"), tuple((hour_12_0, minute, opt(second))))(s)
        .map(|(remaining, result)| {
          (remaining, result.0 + &result.1 + &result.2.unwrap_or_else(|| "".to_string()))
        }),
    _ => preceded(is_a("+-"), tuple((hour_12_0, tag(":"), minute, opt(tuple((tag(":"), second))))))(s)
        .map(|(remaining, result)| {
          let seconds = match &result.3 {
            Some((c, s)) => c.to_string() + s,
            None => "".to_string()
          };
          (remaining, result.0 + result.1 + &result.2 + &seconds)
        })
  }
}

fn timezone_long_offset_with_z(s: &str, d: usize) -> IResult<&str, String, DateTimeError<&str>> {
  match tag::<&str, &str, DateTimeError<&str>>("Z")(s) {
    Ok((remaining, result)) => Ok((remaining, result.into())),
    Err(_) => timezone_long_offset(s, d)
  }
}

fn timezone(s: &str, d: usize) -> IResult<&str, String, DateTimeError<&str>> {
  if d < 4 {
    take_while_m_n(3, 4, is_uppercase)(s).and_then(|(remaining, result)| {
      if validate_tz_abbreviation(result) {
        Ok((remaining, result.into()))
      } else {
        Err(Error(DateTimeError::InvalidTimezone(result.to_string())))
      }
    })
  } else {
    Err(Error(DateTimeError::FullTimezonesNotSupported(s.to_string())))
  }
}

fn timezone_id(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  separated_pair(alphanumeric1, char('/'), alphanumeric1)(s).and_then(|(remaining, result)| {
    let tz = format!("{}/{}", result.0, result.1);
    if ZONES.contains(tz.as_str()) {
      Ok((remaining, tz))
    } else {
      Err(Error(DateTimeError::InvalidTimezone(tz)))
    }
  })
}

fn timezone_offset(s: &str, d: usize) -> IResult<&str, String, DateTimeError<&str>> {
  match d {
    1..=3 => preceded(is_a("+-"), tuple((hour_12_0, minute)))(s)
        .map(|(remaining, result)| {
          (remaining, result.0 + &result.1)
        }),
    4 => alt((
          preceded(alt((tag("GMT"), tag("UTC"))), timezone_hour_min),
          tag("GMT"),
          tag("UTC")
        ))(s)
        .map(|(remaining, result)| {
          (remaining, result.into())
        }),
    _ => timezone_hour_min(s)
        .map(|(remaining, result)| {
          (remaining, result.into())
        })
  }
}

fn timezone_offset_gmt(s: &str, d: usize) -> IResult<&str, String, DateTimeError<&str>> {
  match d {
    1..=3 => preceded(alt((tag("GMT"), tag("UTC"))), tuple((is_a("+-"), hour_12_0, opt(preceded(tag(":"), minute)))))(s)
        .map(|(remaining, result)| {
          let minute = match result.2 {
            Some(result) => result,
            None => "".to_string()
          };
          (remaining, result.1 + &minute)
        }),
    _ => preceded(alt((tag("GMT"), tag("UTC"))), tuple((is_a("+-"), hour_12_0, tag(":"), minute)))(s)
        .map(|(remaining, result)| {
          (remaining, result.1 + result.2 + &result.3)
        })
  }
}

fn day_of_week_name(s: &str, count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  if count <= 3 {
    alt((
      tag_no_case("mon"),
      tag_no_case("tue"),
      tag_no_case("wed"),
      tag_no_case("thu"),
      tag_no_case("fri"),
      tag_no_case("sat"),
      tag_no_case("sun")
    ))(s)
  } else {
    alt((
      tag_no_case("monday"),
      tag_no_case("tuesday"),
      tag_no_case("wednesday"),
      tag_no_case("thursday"),
      tag_no_case("friday"),
      tag_no_case("saturday"),
      tag_no_case("sunday")
    ))(s)
  }.map(|(remaining, result)| (remaining, result.into()))
}

fn quarter_num(s: &str, _count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "quarter".into(), 1, 4) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidQuarter(result.to_string())))
    }
  })
}

#[allow(clippy::comparison_chain)]
fn quarter(s: &str, count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  if count < 3 {
    quarter_num(s, count)
  } else if count == 3 {
    alt((tag_no_case("Q1"), tag_no_case("Q2"), tag_no_case("Q3"), tag_no_case("Q4")))(s)
      .map(|(remaining, result)| (remaining, result.into()))
  } else {
    terminated(alt((
      tag_no_case("1st"), tag_no_case("2nd"), tag_no_case("3rd"), tag_no_case("4th")
    )), tag_no_case(" quarter"))(s)
      .map(|(remaining, result)| (remaining, result.into()))
  }
}

/// Parses a string into a vector of date/time tokens
pub fn parse_pattern(s: &str) -> Result<Vec<DateTimePatternToken>, String> {
  match many1(alt((
    era_pattern,
    year_pattern,
    month_pattern,
    day_in_year_pattern,
    day_in_month_pattern,
    quarter_pattern,
    week_in_year_month_pattern,
    day_name_pattern,
    day_of_week_pattern,
    ampm_pattern,
    hour_pattern,
    minute_pattern,
    second_pattern,
    millisecond_pattern,
    nanosecond_pattern,
    millisecond_of_day_pattern,
    nanosecond_of_day_pattern,
    quoted_text_pattern,
    quote_pattern,
    timezone_pattern,
    text_pattern
  )))(s) {
    Ok((remaining, result)) => if !remaining.is_empty() {
      let error = format!("Parsing datetime pattern '{}' failed at text '{}'", s, remaining);
      debug!("{}", error);
      Err(error)
    } else {
      Ok(result)
    },
    Err(err) => {
      let error = format!("Parsing datetime pattern '{}' failed with error - {}", s, err);
      debug!("{}", error);
      Err(error)
    }
  }
}

fn validate_datetime_string(value: &str, pattern_tokens: &[DateTimePatternToken]) -> Result<(), String> {
  let mut buffer = value;
  let mut matched_values = vec![];
  for token in pattern_tokens {
    let result = match token {
      DateTimePatternToken::Era(count) => era(buffer, *count),
      DateTimePatternToken::Year(count) => year(buffer, *count),
      DateTimePatternToken::WeekInYear => week_in_year(buffer),
      DateTimePatternToken::WeekInMonth(from_one) => week_in_month(buffer, *from_one),
      DateTimePatternToken::DayInYear => day_in_year(buffer),
      DateTimePatternToken::DayInMonth => day_in_month(buffer),
      DateTimePatternToken::Month(count) => month(buffer, *count),
      DateTimePatternToken::MonthNum(count) => month_num(buffer, *count),
      DateTimePatternToken::Text(t) => tag(t.as_str())(buffer).map(|(remaining, result)| (remaining, result.into())),
      DateTimePatternToken::DayName(count) => day_of_week_name(buffer, *count),
      DateTimePatternToken::DayOfWeek(count) => day_of_week(buffer, *count),
      DateTimePatternToken::Hour24 => hour_24(buffer),
      DateTimePatternToken::Hour24ZeroBased => hour_24_0(buffer),
      DateTimePatternToken::Hour12 => hour_12(buffer),
      DateTimePatternToken::Hour12ZeroBased => hour_12_0(buffer),
      DateTimePatternToken::Minute => minute(buffer),
      DateTimePatternToken::Second => second(buffer),
      DateTimePatternToken::Millisecond(size) => second_fraction(buffer, *size),
      DateTimePatternToken::Nanosecond(_size) => digit1(buffer).map(|(remaining, result)| (remaining, result.into())),
      DateTimePatternToken::TimezoneName(size) => timezone(buffer, *size),
      DateTimePatternToken::TimezoneId(_size) => timezone_id(buffer),
      DateTimePatternToken::TimezoneOffset(size) => timezone_offset(buffer, *size),
      DateTimePatternToken::TimezoneOffsetGmt(size) => timezone_offset_gmt(buffer, *size),
      DateTimePatternToken::TimezoneOffsetX(size) => timezone_long_offset(buffer, *size),
      DateTimePatternToken::TimezoneOffsetXZZero(size) => timezone_long_offset_with_z(buffer, *size),
      DateTimePatternToken::AmPm => ampm(buffer),
      DateTimePatternToken::QuarterOfYear(count) => quarter(buffer, *count),
      DateTimePatternToken::QuarterOfYearNum(count) => quarter_num(buffer, *count),
      DateTimePatternToken::MillisecondOfDay => digit1(buffer).map(|(remaining, result)| (remaining, result.into())),
      DateTimePatternToken::NanosecondOfDay => digit1(buffer).map(|(remaining, result)| (remaining, result.into())),
    }.map_err(|err| format!("{:?}", err))?;

    matched_values.push((token.clone(), result.1));

    buffer = result.0;
  }

  if !buffer.is_empty() {
    Err(format!("Remaining data after applying pattern {:?}", buffer))
  } else {
    validate_matched_values(&matched_values)
  }
}

fn validate_matched_values(matched_values: &Vec<(DateTimePatternToken, String)>) -> Result<(), String> {
  // check the day of the month
  let day = matched_values.iter()
    .find(|(token, _)| match token {
      DateTimePatternToken::DayInMonth => true,
      _ => false
    });
  let month = matched_values.iter()
    .find(|(token, _)| match token {
      DateTimePatternToken::Month(_) => true,
      _ => false
    });
  let year = matched_values.iter()
    .find(|(token, _)| match token {
      DateTimePatternToken::Year(_) => true,
      _ => false
    });
  if let Some((_, day)) = day {
    // should be safe to unwrap here, as the value should have already been validated
    let day = day.parse::<u8>().unwrap();
    if let Some((_, year)) = year {
      let year = year.parse::<i16>().unwrap();
      if let Some((_, month_str)) = month {
        // should be safe to unwrap here, as the value should have already been validated
        let month = month_value(month_str).unwrap().with_year(year);
        if day > month.total_days() {
          Err(format!("'{}' is not a valid day for month '{}' in year {}", day, month_str, year))
        } else {
          Ok(())
        }
      } else {
        // No month in matched values
        Ok(())
      }
    } else if let Some((_, month_str)) = month {
      // should be safe to unwrap here, as the value should have already been validated
      let month = month_value(month_str).unwrap().with_year(2001);
      if day > month.total_days() {
        Err(format!("'{}' is not a valid day for month '{}'", day, month_str))
      } else {
        Ok(())
      }
    } else {
      // No month or year in matched values
      Ok(())
    }
  } else {
    // No day in matched values
    Ok(())
  }
}

/// Validates the given datetime against the pattern
pub fn validate_datetime(value: &str, format: &str) -> Result<(), String> {
  match parse_pattern(format) {
    Ok(pattern_tokens) => validate_datetime_string(value, &pattern_tokens),
    Err(err) => Err(format!("Error parsing '{}': {:?}", value, err))
  }
}

/// Converts the date time pattern tokens to a chrono formatted string
pub fn to_chrono_pattern(tokens: &[DateTimePatternToken]) -> String {
  let mut buffer = String::new();

  for token in tokens {
    match token {
      DateTimePatternToken::Era(_count) => buffer.push_str("AD"),
      DateTimePatternToken::Year(d) => buffer.push_str(if *d == 2 { "%y" } else { "%Y" }),
      DateTimePatternToken::WeekInYear => buffer.push_str("%U"),
      DateTimePatternToken::WeekInMonth(_) => warn!("Chono does not support week in month"),
      DateTimePatternToken::DayInYear => buffer.push_str("%j"),
      DateTimePatternToken::DayInMonth => buffer.push_str("%d"),
      DateTimePatternToken::Month(d) => buffer.push_str(if *d <= 2 { "%m" } else if *d > 3 { "%B" } else { "%b" }),
      DateTimePatternToken::MonthNum(_d) => buffer.push_str("%m"),
      DateTimePatternToken::Text(t) => buffer.push_str(&str::replace(t, "%", "%%")),
      DateTimePatternToken::DayName(d) => buffer.push_str(if *d > 3 { "%A" } else { "%a" }),
      DateTimePatternToken::DayOfWeek(_d) => buffer.push_str("%u"),
      DateTimePatternToken::Hour24 => buffer.push_str("%H"),
      DateTimePatternToken::Hour24ZeroBased => buffer.push_str("%H"),
      DateTimePatternToken::Hour12 => buffer.push_str("%I"),
      DateTimePatternToken::Hour12ZeroBased => buffer.push_str("%I"),
      DateTimePatternToken::Minute => buffer.push_str("%M"),
      DateTimePatternToken::Second => buffer.push_str("%S"),
      DateTimePatternToken::Millisecond(d) => if *d < 3 {
        // something in chrono panics
        buffer.push_str("%3f")
      } else {
        buffer.push_str(&format!("%{}f", *d))
      },
      DateTimePatternToken::Nanosecond(_d) => buffer.push_str("%f"),
      DateTimePatternToken::TimezoneName(_d) => buffer.push_str("%Z"),
      DateTimePatternToken::TimezoneId(_d) => buffer.push_str("%Z"),
      DateTimePatternToken::TimezoneOffset(_d) => buffer.push_str("%z"),
      DateTimePatternToken::TimezoneOffsetX(_d) => buffer.push_str("%:z"),
      DateTimePatternToken::TimezoneOffsetXZZero(_d) => buffer.push_str("%:z"),
      DateTimePatternToken::AmPm => buffer.push_str("%p"),
      _ => warn!("Chono does not support {:?}", token)
    };
  }

  buffer
}

/// Generates a date/time string from the current system clock using the provided format string
pub fn generate_string(format: &str) -> Result<String, String> {
  trace!("generating date/time from '{}'", format);
  match parse_pattern(format) {
    Ok(pattern_tokens) => {
      trace!("parsed date/time patterns: {:?}", pattern_tokens);
      let chrono_pattern = to_chrono_pattern(&pattern_tokens);
      trace!("Chrono pattern: {}", chrono_pattern);
      Ok(Local::now().format(chrono_pattern.as_str()).to_string())
    },
    Err(err) => {
      error!("Error parsing '{}': {:?}", format, err);
      Err(format!("Error parsing '{}': {:?}", format, err))
    }
  }
}

fn validate_tz_abbreviation(tz: &str) -> bool {
  ZONES_ABBR.contains_key(tz)
}

fn month_value(value: &str) -> anyhow::Result<gregorian::Month> {
  match value.to_lowercase().as_str() {
    "january" | "jan" => Ok(Month::January),
    "february" | "feb" => Ok(Month::February),
    "march" | "mar" => Ok(Month::March),
    "april" | "apr" => Ok(Month::April),
    "may" => Ok(Month::May),
    "june" | "jun" => Ok(Month::June),
    "july" | "jul" => Ok(Month::July),
    "august" | "aug" => Ok(Month::August),
    "september" | "sep" => Ok(Month::September),
    "october" | "oct" => Ok(Month::October),
    "november" | "nov" => Ok(Month::November),
    "december" | "dec" => Ok(Month::December),
    _ => if let Ok(month) = value.parse::<u8>() {
      Month::new(month).map_err(|_| anyhow!("'{}' is not a valid month value", value))
    } else {
      Err(anyhow!("'{}' is not a valid month value", value))
    }
  }
}

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;

  use super::*;

  #[test]
  fn parse_date_and_time() {
    expect!(validate_datetime("2001-01-02", "yyyy-MM-dd")).to(be_ok());
    expect!(validate_datetime("2001-01-02 12:33:45", "yyyy-MM-dd HH:mm:ss")).to(be_ok());

    expect!(validate_datetime("2001-13-02", "yyyy-MM-dd")).to(be_err());
    expect!(validate_datetime("2001-01-02 25:33:45", "yyyy-MM-dd HH:mm:ss")).to(be_err());

    expect!(validate_datetime("2001.07.04 AD at 12:08:56 PDT", "yyyy.MM.dd G 'at' HH:mm:ss z")).to(be_ok());
    expect!(validate_datetime("Wed, Jul 4, '01", "EEE, MMM d, ''yy")).to(be_ok());
    expect!(validate_datetime("12:08 PM", "h:mm a")).to(be_ok());
//    "hh 'o''clock' a, zzzz"	12 o'clock PM, Pacific Daylight Time
    expect!(validate_datetime("0:08 PM, AEST", "K:mm a, z")).to(be_ok());
    expect!(validate_datetime("02001.July.04 AD 12:08 PM", "yyyyy.MMMMM.dd G hh:mm a")).to(be_ok());
    expect!(validate_datetime("Wed, 4 Jul 2001 12:08:56 -0700", "EEE, d MMM yyyy HH:mm:ss Z")).to(be_ok());
    expect!(validate_datetime("010704120856-0700", "yyMMddHHmmssZ")).to(be_ok());
    expect!(validate_datetime("2001-07-04T12:08:56.235-0700", "yyyy-MM-dd'T'HH:mm:ss.SSSZ")).to(be_ok());
    expect!(validate_datetime("2001-07-04T12:08:56.235Z", "yyyy-MM-dd'T'HH:mm:ss.SSSX")).to(be_ok());
    expect!(validate_datetime("2001-07-04T12:08:56.235-07:00", "yyyy-MM-dd'T'HH:mm:ss.SSSXXX")).to(be_ok());
    expect!(validate_datetime("2001-W27-3", "YYYY-'W'ww-u")).to(be_ok());

    expect!(validate_datetime("2020-01-01T10:00+01:00[Europe/Warsaw]", "yyyy-MM-dd'T'HH:mmXXX'['VV']'")).to(be_ok());
  }

  #[test]
  fn parse_date_and_time_with_invalid_dates() {
    // Invalid Febuary date
    expect!(validate_datetime("2000-02-30", "yyyy-MM-dd")).to(be_err()
      .value("'30' is not a valid day for month '02' in year 2000"));
    // Invalid June date
    expect!(validate_datetime("2000-06-31", "yyyy-MM-dd")).to(be_err());
    // Not a leap year
    expect!(validate_datetime("2001-02-29", "yyyy-MM-dd")).to(be_err());

    //    September 1752
    // Su Mo Tu We Th Fr Sa
    //        1  2 14 15 16
    // 17 18 19 20 21 22 23
    // 24 25 26 27 28 29 30
    // Nuff said, but is not supported by the Rust crates as yet
    //expect!(validate_datetime("1752-09-03", "yyyy-MM-dd")).to(be_err());
  }

  #[test]
  fn parse_era() {
    expect!(parse_pattern("G")).to(
      be_ok().value(vec![DateTimePatternToken::Era(1)]));
    expect!(parse_pattern("GG")).to(
      be_ok().value(vec![DateTimePatternToken::Era(2)]));
    expect!(parse_pattern("GGGGG")).to(
      be_ok().value(vec![DateTimePatternToken::Era(5)]));

    let result = parse_pattern("GGGGGG");
    expect!(result.clone()).to(be_err());
    expect!(result.unwrap_err().contains("Too many pattern letters for Era")).to(be_true());


    expect!(validate_datetime("ad", "G")).to(be_ok());
    expect!(validate_datetime("AD", "GG")).to(be_ok());
    expect!(validate_datetime("bc", "GGG")).to(be_ok());
    expect!(validate_datetime("BC", "G")).to(be_ok());
    expect!(validate_datetime("BX", "G")).to(be_err());
  }

  #[test]
  fn parse_ampm() {
    expect!(parse_pattern("a")).to(
      be_ok().value(vec![DateTimePatternToken::AmPm]));
    expect!(parse_pattern("aa")).to(be_err());
    expect!(parse_pattern("aaaa")).to(be_err());

    expect!(validate_datetime("am", "a")).to(be_ok());
    expect!(validate_datetime("AM", "a")).to(be_ok());
    expect!(validate_datetime("pm", "a")).to(be_ok());
    expect!(validate_datetime("PM", "a")).to(be_ok());
    expect!(validate_datetime("PX", "a")).to(be_err());
  }

  #[test]
  fn parse_year() {
    expect!(parse_pattern("y")).to(
      be_ok().value(vec![DateTimePatternToken::Year(1)]));
    expect!(parse_pattern("u")).to(
      be_ok().value(vec![DateTimePatternToken::Year(1)]));
    expect!(parse_pattern("yy")).to(
      be_ok().value(vec![DateTimePatternToken::Year(2)]));
    expect!(parse_pattern("yyyy")).to(
      be_ok().value(vec![DateTimePatternToken::Year(4)]));
    expect!(parse_pattern("YYyy")).to(
      be_ok().value(vec![DateTimePatternToken::Year(2), DateTimePatternToken::Year(2)]));

    expect!(validate_datetime("2000", "yyyy")).to(be_ok());
    expect!(validate_datetime("200000", "yyyyyy")).to(be_ok());
    expect!(validate_datetime("20", "yy")).to(be_ok());
    expect!(validate_datetime("2000", "YYYY")).to(be_ok());
    expect!(validate_datetime("20", "YY")).to(be_ok());
    expect!(validate_datetime("20", "yyyy")).to(be_ok());
    expect!(validate_datetime("", "yyyy")).to(be_err());
  }

  #[test]
  fn parse_month() {
    expect!(parse_pattern("M")).to(
      be_ok().value(vec![DateTimePatternToken::Month(1)]));
    expect!(parse_pattern("MM")).to(
      be_ok().value(vec![DateTimePatternToken::Month(2)]));
    expect!(parse_pattern("LLL")).to(
      be_ok().value(vec![DateTimePatternToken::MonthNum(3)]));
    expect!(parse_pattern("MMMMMM")).to(be_err());

    expect!(validate_datetime("jan", "M")).to(be_err());
    expect!(validate_datetime("jan", "MMM")).to(be_ok());
    expect!(validate_datetime("october", "MMM")).to(be_err());
    expect!(validate_datetime("December", "MMMM")).to(be_ok());
    expect!(validate_datetime("December", "L")).to(be_err());
    expect!(validate_datetime("01", "L")).to(be_ok());
    expect!(validate_datetime("10", "MM")).to(be_ok());
    expect!(validate_datetime("100", "MM")).to(be_err());
    expect!(validate_datetime("100", "LL")).to(be_err());
    expect!(validate_datetime("13", "MM")).to(be_err());
    expect!(validate_datetime("31", "MM")).to(be_err());
    expect!(validate_datetime("00", "MM")).to(be_err());
    expect!(validate_datetime("", "MMM")).to(be_err());
  }

  #[test]
  fn parse_text() {
    expect!(parse_pattern("'ello'")).to(
      be_ok().value(vec![DateTimePatternToken::Text("ello".chars().collect())]));
    expect!(parse_pattern("'dd-MM-yyyy'")).to(
      be_ok().value(vec![DateTimePatternToken::Text("dd-MM-yyyy".chars().collect())]));
    expect!(parse_pattern("''")).to(
      be_ok().value(vec![DateTimePatternToken::Text("'".chars().collect())]));
    expect!(parse_pattern("'dd-''MM''-yyyy'")).to(
      be_ok().value(vec![DateTimePatternToken::Text("dd-'MM'-yyyy".chars().collect())]));

    expect!(validate_datetime("ello", "'ello'")).to(be_ok());
    expect!(validate_datetime("elo", "'ello'")).to(be_err());
    expect!(validate_datetime("dd-MM-yyyy", "'dd-MM-yyyy'")).to(be_ok());
  }

  #[test]
  fn parse_week_number() {
    expect!(parse_pattern("wW")).to(
      be_ok().value(vec![DateTimePatternToken::WeekInYear, DateTimePatternToken::WeekInMonth(true)]));
    expect!(parse_pattern("www")).to(be_err());
    expect!(parse_pattern("WW")).to(
      be_ok().value(vec![DateTimePatternToken::WeekInMonth(true)]));
    expect!(parse_pattern("F")).to(
      be_ok().value(vec![DateTimePatternToken::WeekInMonth(false)]));

    expect!(validate_datetime("12", "w")).to(be_ok());
    expect!(validate_datetime("3", "WW")).to(be_ok());
    expect!(validate_datetime("57", "ww")).to(be_err());
    expect!(validate_datetime("0", "W")).to(be_err());
    expect!(validate_datetime("0", "F")).to(be_ok());
  }

  #[test]
  fn parse_day_number() {
    expect!(parse_pattern("dD")).to(
      be_ok().value(vec![DateTimePatternToken::DayInMonth, DateTimePatternToken::DayInYear]));
    expect!(parse_pattern("dd")).to(
      be_ok().value(vec![DateTimePatternToken::DayInMonth]));
    expect!(parse_pattern("DDD")).to(
      be_ok().value(vec![DateTimePatternToken::DayInYear]));
    expect!(parse_pattern("ddd")).to(be_err());

    expect!(validate_datetime("12", "d")).to(be_ok());
    expect!(validate_datetime("03", "DD")).to(be_ok());
    expect!(validate_datetime("32", "dd")).to(be_err());
    expect!(validate_datetime("0", "D")).to(be_err());
    expect!(validate_datetime("357", "D")).to(be_err());
  }

  #[test]
  fn parse_day_of_week() {
    expect!(parse_pattern("c")).to(
      be_ok().value(vec![DateTimePatternToken::DayOfWeek(1)]));
    expect!(parse_pattern("EE")).to(
      be_ok().value(vec![DateTimePatternToken::DayName(2)]));
    expect!(parse_pattern("ee")).to(
      be_ok().value(vec![DateTimePatternToken::DayOfWeek(2)]));

    expect!(validate_datetime("7", "c")).to(be_ok());
    expect!(validate_datetime("Tue", "EEE")).to(be_ok());
    expect!(validate_datetime("Tuesday", "EEEE")).to(be_ok());
    expect!(validate_datetime("3", "E")).to(be_err());
    expect!(validate_datetime("3", "e")).to(be_ok());
    expect!(validate_datetime("32", "ee")).to(be_err());
    expect!(validate_datetime("0", "c")).to(be_err());
  }

  #[test]
  fn parse_hour() {
    expect!(parse_pattern("k")).to(
      be_ok().value(vec![DateTimePatternToken::Hour24]));
    expect!(parse_pattern("KK")).to(
      be_ok().value(vec![DateTimePatternToken::Hour12ZeroBased]));
    expect!(parse_pattern("hh")).to(
      be_ok().value(vec![DateTimePatternToken::Hour12]));
    expect!(parse_pattern("HH")).to(
      be_ok().value(vec![DateTimePatternToken::Hour24ZeroBased]));
    expect!(parse_pattern("HHHH")).to(be_err());

    expect!(validate_datetime("11", "k")).to(be_ok());
    expect!(validate_datetime("11", "KK")).to(be_ok());
    expect!(validate_datetime("11", "hh")).to(be_ok());
    expect!(validate_datetime("11", "H")).to(be_ok());

    expect!(validate_datetime("25", "kk")).to(be_err());
    expect!(validate_datetime("0", "k")).to(be_err());
    expect!(validate_datetime("0", "KK")).to(be_ok());
    expect!(validate_datetime("12", "KK")).to(be_err());
    expect!(validate_datetime("12", "h")).to(be_ok());
    expect!(validate_datetime("0", "hh")).to(be_err());
    expect!(validate_datetime("0", "H")).to(be_ok());
    expect!(validate_datetime("23", "H")).to(be_ok());
    expect!(validate_datetime("24", "HH")).to(be_err());
  }

  #[test]
  fn parse_minute_and_second() {
    expect!(parse_pattern("m")).to(
      be_ok().value(vec![DateTimePatternToken::Minute]));
    expect!(parse_pattern("s")).to(
      be_ok().value(vec![DateTimePatternToken::Second]));
    expect!(parse_pattern("SSS")).to(
      be_ok().value(vec![DateTimePatternToken::Millisecond(3)]));
    expect!(parse_pattern("A")).to(
      be_ok().value(vec![DateTimePatternToken::MillisecondOfDay]));
    expect!(parse_pattern("n")).to(
      be_ok().value(vec![DateTimePatternToken::Nanosecond(1)]));
    expect!(parse_pattern("N")).to(
      be_ok().value(vec![DateTimePatternToken::NanosecondOfDay]));

    expect!(validate_datetime("12", "m")).to(be_ok());
    expect!(validate_datetime("03", "ss")).to(be_ok());
    expect!(validate_datetime("030", "SSS")).to(be_ok());
    expect!(validate_datetime("123456", "SSSSSS")).to(be_ok());
    expect!(validate_datetime("35392790", "A")).to(be_ok());
    expect!(validate_datetime("35392790", "n")).to(be_ok());
    expect!(validate_datetime("60", "m")).to(be_err());
    expect!(validate_datetime("61", "s")).to(be_err());
    expect!(validate_datetime("1000", "SS")).to(be_err());
    expect!(validate_datetime("1000000", "SSSSSS")).to(be_err());
  }

  #[test]
  fn parse_timezone() {
    expect!(parse_pattern("x")).to(
      be_ok().value(vec![DateTimePatternToken::TimezoneOffsetX(1)]));
    expect!(parse_pattern("Z")).to(
      be_ok().value(vec![DateTimePatternToken::TimezoneOffset(1)]));
    expect!(parse_pattern("XXX")).to(
      be_ok().value(vec![DateTimePatternToken::TimezoneOffsetXZZero(3)]));
    expect!(parse_pattern("OOOO")).to(
      be_ok().value(vec![DateTimePatternToken::TimezoneOffsetGmt(4)]));

    expect!(validate_datetime("-0700", "Z")).to(be_ok());
    expect!(validate_datetime("1100", "ZZZZ")).to(be_err());
    expect!(validate_datetime("GMT+10:00", "ZZZZ")).to(be_ok());
    expect!(validate_datetime("+1030", "Z")).to(be_ok());
    expect!(validate_datetime("-2400", "Z")).to(be_err());
    expect!(validate_datetime("2361", "Z")).to(be_err());
    expect!(validate_datetime("Z", "Z")).to(be_err());
    expect!(validate_datetime("GMT", "ZZZZ")).to(be_ok());
    expect!(validate_datetime("+10:00", "ZZZZZ")).to(be_ok());

    expect!(validate_datetime("Z", "X")).to(be_ok());
    expect!(validate_datetime("Z", "x")).to(be_err());
    expect!(validate_datetime("-0730", "X")).to(be_ok());
    expect!(validate_datetime("+08", "X")).to(be_ok());
    expect!(validate_datetime("-0730", "x")).to(be_ok());
    expect!(validate_datetime("+0800", "x")).to(be_ok());
    expect!(validate_datetime("-0730", "XX")).to(be_ok());
    expect!(validate_datetime("+0800", "xx")).to(be_ok());
    expect!(validate_datetime("-07:30", "XXX")).to(be_ok());
    expect!(validate_datetime("+08:00", "xxx")).to(be_ok());
    expect!(validate_datetime("-0730", "XXXX")).to(be_ok());
    expect!(validate_datetime("+0800", "xxxx")).to(be_ok());
    expect!(validate_datetime("-073000", "XXXX")).to(be_ok());
    expect!(validate_datetime("+080000", "xxxx")).to(be_ok());
    expect!(validate_datetime("-07:30:00", "XXXXX")).to(be_ok());
    expect!(validate_datetime("+08:00:00", "xxxxx")).to(be_ok());

    expect!(validate_datetime("1100", "XX")).to(be_err());
    expect!(validate_datetime("1100", "xx")).to(be_err());
    expect!(validate_datetime("+10", "XX")).to(be_err());
    expect!(validate_datetime("+10", "xx")).to(be_err());
    expect!(validate_datetime("-0730", "XXX")).to(be_err());
    expect!(validate_datetime("+0800", "xxx")).to(be_err());
    expect!(validate_datetime("-07:30", "XXXX")).to(be_err());
    expect!(validate_datetime("+08:00", "xxxx")).to(be_err());
    expect!(validate_datetime("-073000", "XXXXX")).to(be_err());
    expect!(validate_datetime("+080000", "xxxxx")).to(be_err());

    expect!(validate_datetime("GMT-7", "O")).to(be_ok());
    expect!(validate_datetime("UTC+10", "O")).to(be_ok());
    expect!(validate_datetime("UTC+9:30", "O")).to(be_ok());
    expect!(validate_datetime("GMT+08:00", "OOOO")).to(be_ok());
    expect!(validate_datetime("GMT+08", "OOOO")).to(be_err());

    // expect!(validate_datetime("AEST", "z")).to(be_ok());
    // expect!(validate_datetime("BST", "z")).to(be_ok());
    // expect!(validate_datetime("UTC", "z")).to(be_ok());
    // expect!(validate_datetime("aest", "z")).to(be_err());
    // expect!(validate_datetime("AEST", "zzzz")).to(be_err());
  }

  #[test]
  fn to_chrono_pattern_test() {
    expect!(to_chrono_pattern(&parse_pattern("yyyy-MM-dd").unwrap())).to(be_equal_to("%Y-%m-%d"));
    expect!(to_chrono_pattern(&parse_pattern("yyyy-MM-dd HH:mm:ss").unwrap())).to(be_equal_to("%Y-%m-%d %H:%M:%S"));
    expect!(to_chrono_pattern(&parse_pattern("EEE, MMM d, ''yy").unwrap())).to(be_equal_to("%a, %b %d, \'%y"));
    expect!(to_chrono_pattern(&parse_pattern("h:mm a").unwrap())).to(be_equal_to("%I:%M %p"));
    expect!(to_chrono_pattern(&parse_pattern("hh 'o''clock' a, z").unwrap())).to(be_equal_to("%I o'clock %p, %Z"));
    expect!(to_chrono_pattern(&parse_pattern("yyyyy.MMMMM.dd GGG hh:mm a").unwrap())).to(be_equal_to("%Y.%B.%d AD %I:%M %p"));
    expect!(to_chrono_pattern(&parse_pattern("EEE, d MMM yyyy HH:mm:ss Z").unwrap())).to(be_equal_to("%a, %d %b %Y %H:%M:%S %z"));
    expect!(to_chrono_pattern(&parse_pattern("yyMMddHHmmssZ").unwrap())).to(be_equal_to("%y%m%d%H%M%S%z"));
    expect!(to_chrono_pattern(&parse_pattern("yyyy-MM-dd'T'HH:mm:ss.SSSZ").unwrap())).to(be_equal_to("%Y-%m-%dT%H:%M:%S.%3f%z"));
    expect!(to_chrono_pattern(&parse_pattern("yyyy-MM-dd'T'HH:mm:ss.SSSXXX").unwrap())).to(be_equal_to("%Y-%m-%dT%H:%M:%S.%3f%:z"));
    expect!(to_chrono_pattern(&parse_pattern("YYYY-'W'ww-e").unwrap())).to(be_equal_to("%Y-W%U-%u"));
  }

  #[test]
  fn parse_quarter() {
    expect!(parse_pattern("Q")).to(
      be_ok().value(vec![DateTimePatternToken::QuarterOfYear(1)]));
    expect!(parse_pattern("QQ")).to(
      be_ok().value(vec![DateTimePatternToken::QuarterOfYear(2)]));
    expect!(parse_pattern("QQQ")).to(
      be_ok().value(vec![DateTimePatternToken::QuarterOfYear(3)]));
    expect!(parse_pattern("QQQQQQ")).to(be_err());
    expect!(parse_pattern("q")).to(
      be_ok().value(vec![DateTimePatternToken::QuarterOfYearNum(1)]));
    expect!(parse_pattern("qqq")).to(be_err());

    expect!(validate_datetime("2", "Q")).to(be_ok());
    expect!(validate_datetime("2", "q")).to(be_ok());
    expect!(validate_datetime("02", "QQ")).to(be_ok());
    expect!(validate_datetime("02", "qq")).to(be_ok());
    expect!(validate_datetime("Q2", "QQ")).to(be_err());
    expect!(validate_datetime("Q2", "QQQ")).to(be_ok());
    expect!(validate_datetime("Q2", "qq")).to(be_err());
    expect!(validate_datetime("2nd quarter", "QQQQ")).to(be_ok());
    expect!(validate_datetime("5th quarter", "QQQQ")).to(be_err());
  }

  #[test]
  fn timezone_abbreviations() {
    expect!(validate_tz_abbreviation("AEST")).to(be_true());
    expect!(validate_tz_abbreviation("AEDT")).to(be_true());
    expect!(validate_tz_abbreviation("XXX")).to(be_false());
  }
}
