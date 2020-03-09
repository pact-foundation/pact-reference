use nom::types::CompleteStr;
use nom::*;
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateTimePatternToken {
  Era,
  Year(usize),
  Month(usize),
  Text(Vec<char>),
  WeekInYear,
  WeekInMonth,
  DayInYear,
  DayInMonth,
  DayOfWeekInMonth,
  DayName(usize),
  DayOfWeek,
  AmPm,
  Hour24,
  Hour24ZeroBased,
  Hour12,
  Hour12ZeroBased,
  Minute,
  Second,
  Millisecond,
  Timezone,
  Rfc822Timezone,
  Iso8601Timezone
}

fn is_digit(ch: char) -> bool {
  ch.is_ascii_digit()
}

fn validate_number(m: CompleteStr, num_type: String, lower: usize, upper: usize) -> Result<CompleteStr, String> {
  match m.0.parse::<usize>() {
    Ok(v) => if v >= lower && v <= upper {
      Ok(m)
    } else {
      Err(format!("Invalid {} {}", num_type, v))
    },
    Err(err) => Err(format!("{}", err))
  }
}

fn validate_month(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "month".into(), 1, 12)
}

fn validate_week_in_year(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "week in year".into(), 1, 56)
}

fn validate_week_in_month(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "week in month".into(), 1, 5)
}

fn validate_day_in_year(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "day in year".into(), 1, 356)
}

fn validate_day_in_month(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "day in month".into(), 1, 31)
}

fn validate_day_of_week(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "day of week".into(), 1, 7)
}

fn validate_hour_24(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "hour (24)".into(), 1, 24)
}

fn validate_hour_24_0(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "hour (24 zero-based)".into(), 0, 23)
}

fn validate_hour_12(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "hour".into(), 1, 12)
}

fn validate_hour_12_0(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "hour (zero-based)".into(), 0, 11)
}

fn validate_minute(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "minute".into(), 0, 59)
}

fn validate_second(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "second".into(), 0, 59)
}

fn validate_millisecond(m: CompleteStr) -> Result<CompleteStr, String> {
  validate_number(m, "millisecond".into(), 0, 999)
}

named!(era_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::Era, many1!(char!('G'))));
named!(ampm_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::AmPm, many1!(char!('a'))));
named!(week_in_year_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::WeekInYear, many1!(char!('w'))));
named!(week_in_month_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::WeekInMonth, many1!(char!('W'))));
named!(day_in_year_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::DayInYear, many1!(char!('D'))));
named!(day_in_month_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::DayInMonth, many1!(char!('d'))));
named!(day_of_week_in_month_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::DayOfWeekInMonth, many1!(char!('F'))));
named!(day_name_pattern <CompleteStr, DateTimePatternToken>, do_parse!(d: is_a!("E") >> (DateTimePatternToken::DayName(d.len()))));
named!(day_of_week_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::DayOfWeek, many1!(char!('u'))));
named!(year_pattern <CompleteStr, DateTimePatternToken>, do_parse!(y: is_a!("yY") >> (DateTimePatternToken::Year(y.len())) ));
named!(month_pattern <CompleteStr, DateTimePatternToken>, do_parse!(m: is_a!("ML") >> (DateTimePatternToken::Month(m.len()))));
named!(text_pattern <CompleteStr, DateTimePatternToken>, do_parse!(
  t: many1!(none_of!("GyYMLwWdDFEu'akKhHmsSzZX"))
  >> (DateTimePatternToken::Text(t))
));
named!(quoted_text_pattern <CompleteStr, DateTimePatternToken>, do_parse!(
  char!('\'')
  >> t: many1!(alt!(tag!("''") | is_not!("'")))
  >> char!('\'')
  >> (DateTimePatternToken::Text(t.iter()
    .map(|s| s.chars().coalesce(|x, y| if x == '\'' && y == '\'' { Ok('\'') } else { Err((x, y)) }).collect::<String>())
    .join("").chars().collect()))
));
named!(quote_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::Text("'".chars().collect()), tag!("''")));
named!(hour_24_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::Hour24, many1!(char!('k'))));
named!(hour_24_0_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::Hour24ZeroBased, many1!(char!('H'))));
named!(hour_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::Hour12, many1!(char!('h'))));
named!(hour_0_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::Hour12ZeroBased, many1!(char!('K'))));
named!(minute_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::Minute, many1!(char!('m'))));
named!(second_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::Second, many1!(char!('s'))));
named!(millisecond_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::Millisecond, many1!(char!('S'))));
named!(timezone_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::Timezone, many1!(char!('z'))));
named!(rfc_timezone_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::Rfc822Timezone, many1!(char!('Z'))));
named!(iso_timezone_pattern <CompleteStr, DateTimePatternToken>, value!(DateTimePatternToken::Iso8601Timezone, many1!(char!('X'))));
named!(pub parse_pattern <CompleteStr, Vec<DateTimePatternToken> >, do_parse!(
  v: many0!(alt!(
    era_pattern |
    year_pattern |
    month_pattern |
    week_in_year_pattern |
    week_in_month_pattern |
    day_in_year_pattern |
    day_in_month_pattern |
    day_of_week_in_month_pattern |
    day_name_pattern |
    day_of_week_pattern |
    ampm_pattern |
    hour_24_pattern |
    hour_24_0_pattern |
    hour_pattern |
    hour_0_pattern |
    minute_pattern |
    second_pattern |
    millisecond_pattern |
    timezone_pattern |
    rfc_timezone_pattern |
    iso_timezone_pattern |
    quoted_text_pattern |
    quote_pattern |
    text_pattern)) >> (v)
));

named!(era <CompleteStr, CompleteStr>, alt!(tag_no_case!("ad") | tag_no_case!("bc")));
named!(ampm <CompleteStr, CompleteStr>, alt!(tag_no_case!("am") | tag_no_case!("pm")));
named_args!(year(digits: usize) <CompleteStr, CompleteStr>, take_while_m_n!(1, digits, is_digit));
named!(month_text <CompleteStr, CompleteStr>, alt!(
  tag_no_case!("january")   | tag_no_case!("jan") |
  tag_no_case!("february")  | tag_no_case!("feb") |
  tag_no_case!("march")     | tag_no_case!("mar") |
  tag_no_case!("april")     | tag_no_case!("apr") |
  tag_no_case!("may")       | tag_no_case!("may") |
  tag_no_case!("june")      | tag_no_case!("jun") |
  tag_no_case!("july")      | tag_no_case!("jul") |
  tag_no_case!("august")    | tag_no_case!("aug") |
  tag_no_case!("september") | tag_no_case!("sep") |
  tag_no_case!("october")   | tag_no_case!("oct") |
  tag_no_case!("november")  | tag_no_case!("nov") |
  tag_no_case!("december")  | tag_no_case!("dec")
));
named!(month_num <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 2, is_digit), validate_month));
named!(month <CompleteStr, CompleteStr>, alt!(month_text | month_num));
named!(week_in_year <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 2, is_digit), validate_week_in_year));
named!(week_in_month <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 2, is_digit), validate_week_in_month));
named!(day_in_year <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 2, is_digit), validate_day_in_year));
named!(day_in_month <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 2, is_digit), validate_day_in_month));
named!(day_of_week <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 1, is_digit), validate_day_of_week));
named!(hour_24 <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 2, is_digit), validate_hour_24));
named!(hour_24_0 <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 2, is_digit), validate_hour_24_0));
named!(hour_12 <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 2, is_digit), validate_hour_12));
named!(hour_12_0 <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 2, is_digit), validate_hour_12_0));
named!(minute <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 2, is_digit), validate_minute));
named!(second <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 2, is_digit), validate_second));
named!(millisecond <CompleteStr, CompleteStr>, map_res!(take_while_m_n!(1, 3, is_digit), validate_millisecond));
named!(timezone <CompleteStr, CompleteStr>,
  alt!(
    do_parse!(alt!(tag!("GMT") | tag!("UTC")) >> is_a!("+-") >> hour_24_0 >> tag!(":") >> minute >> (CompleteStr(""))) |
    tag!("GMT") | tag!("UTC")
  )
);
named!(rfc_timezone <CompleteStr, CompleteStr>, do_parse!(is_a!("+-") >> hour_24_0 >> minute >> (CompleteStr(""))));
named!(iso_timezone <CompleteStr, CompleteStr>, alt!(tag!("Z") | do_parse!(is_a!("+-") >> hour_12_0 >> opt!(tag!(":")) >> opt!(minute) >> (CompleteStr("")))));
named_args!(text<'a>(t: &'a Vec<char>) <CompleteStr<'a>, CompleteStr<'a>>, tag!(t.iter().collect::<String>().as_str()));
named!(day_of_week_name <CompleteStr, CompleteStr>, alt!(
  tag_no_case!("sunday")    | tag_no_case!("sun") |
  tag_no_case!("monday")    | tag_no_case!("mon") |
  tag_no_case!("tuesday")   | tag_no_case!("tue") |
  tag_no_case!("wednesday") | tag_no_case!("wed") |
  tag_no_case!("thursday")  | tag_no_case!("thu") |
  tag_no_case!("friday")    | tag_no_case!("fri") |
  tag_no_case!("saturday")  | tag_no_case!("sat")
));

fn validate_datetime_string<'a>(value: &String, pattern_tokens: &Vec<DateTimePatternToken>) -> Result<(), String> {
  let mut buffer = CompleteStr(&value);
  for token in pattern_tokens {
    let result = match token {
      DateTimePatternToken::Era => era(buffer),
      DateTimePatternToken::Year(d) => year(buffer, d.clone()),
      DateTimePatternToken::WeekInYear => week_in_year(buffer),
      DateTimePatternToken::WeekInMonth => week_in_month(buffer),
      DateTimePatternToken::DayInYear => day_in_year(buffer),
      DateTimePatternToken::DayInMonth => day_in_month(buffer),
      DateTimePatternToken::Month(_m) => month(buffer),
      DateTimePatternToken::Text(t) => text(buffer, t),
      DateTimePatternToken::DayOfWeekInMonth => digit1(buffer),
      DateTimePatternToken::DayName(_d) => day_of_week_name(buffer),
      DateTimePatternToken::DayOfWeek => day_of_week(buffer),
      DateTimePatternToken::Hour24 => hour_24(buffer),
      DateTimePatternToken::Hour24ZeroBased => hour_24_0(buffer),
      DateTimePatternToken::Hour12 => hour_12(buffer),
      DateTimePatternToken::Hour12ZeroBased => hour_12_0(buffer),
      DateTimePatternToken::Minute => minute(buffer),
      DateTimePatternToken::Second => second(buffer),
      DateTimePatternToken::Millisecond => millisecond(buffer),
      DateTimePatternToken::Timezone => timezone(buffer),
      DateTimePatternToken::Rfc822Timezone => rfc_timezone(buffer),
      DateTimePatternToken::Iso8601Timezone => iso_timezone(buffer),
      DateTimePatternToken::AmPm => ampm(buffer)
    }.map_err(|err| format!("{:?}", err))?;
    buffer = result.0;
  }

  if buffer.len() > 0 {
    Err(format!("Remaining data after applying pattern {:?}", buffer))
  } else {
    Ok(())
  }
}

pub fn validate_datetime(value: &String, format: &String) -> Result<(), String> {
  match parse_pattern(CompleteStr(format.as_str())) {
    Ok(pattern_tokens) => validate_datetime_string(value, &pattern_tokens.1),
    Err(err) => Err(format!("{:?}", err))
  }
}

pub fn to_chrono_pattern(tokens: &Vec<DateTimePatternToken>) -> String {
  let mut buffer = String::new();

  for token in tokens {
    buffer.push_str(match token {
      DateTimePatternToken::Era => "AD".into(),
      DateTimePatternToken::Year(d) => if *d == 2 { "%y".into() } else { "%Y".into() },
      DateTimePatternToken::WeekInYear => "%U".into(),
      DateTimePatternToken::WeekInMonth => {
        log::warn!("Chono does not support week in month");
        "".into()
      },
      DateTimePatternToken::DayInYear => "%j".into(),
      DateTimePatternToken::DayInMonth => "%d".into(),
      DateTimePatternToken::Month(d) => if *d <= 2 { "%m".into() } else if *d > 3 { "%B".into() } else { "%b".into() },
      DateTimePatternToken::Text(t) => t.iter().join("").replace("%", "%%").to_owned(),
      DateTimePatternToken::DayOfWeekInMonth => {
        log::warn!("Chono does not support day of week in month");
        "".into()
      },
      DateTimePatternToken::DayName(d) => if *d > 3 { "%A".into() } else { "%a".into() },
      DateTimePatternToken::DayOfWeek => "%u".into(),
      DateTimePatternToken::Hour24 => "%H".into(),
      DateTimePatternToken::Hour24ZeroBased => "%H".into(),
      DateTimePatternToken::Hour12 => "%I".into(),
      DateTimePatternToken::Hour12ZeroBased => "%I".into(),
      DateTimePatternToken::Minute => "%M".into(),
      DateTimePatternToken::Second => "%S".into(),
      DateTimePatternToken::Millisecond => "%3f".into(),
      DateTimePatternToken::Timezone => "%:z".into(),
      DateTimePatternToken::Rfc822Timezone => "%z".into(),
      DateTimePatternToken::Iso8601Timezone => "%:z".into(),
      DateTimePatternToken::AmPm => "%p".into()
    }.as_str());
  }

  buffer
}

#[cfg(test)]
mod tests {
  use super::*;
  use expectest::prelude::*;
  use expectest::expect;

  #[test]
  fn parse_date_and_time() {
    expect!(validate_datetime(&"2001-01-02".into(), &"yyyy-MM-dd".into())).to(be_ok());
    expect!(validate_datetime(&"2001-01-02 12:33:45".into(), &"yyyy-MM-dd HH:mm:ss".into())).to(be_ok());

    expect!(validate_datetime(&"2001-13-02".into(), &"yyyy-MM-dd".into())).to(be_err());
    expect!(validate_datetime(&"2001-01-02 25:33:45".into(), &"yyyy-MM-dd HH:mm:ss".into())).to(be_err());

//    "yyyy.MM.dd G 'at' HH:mm:ss z"	2001.07.04 AD at 12:08:56 PDT
    expect!(validate_datetime(&"Wed, Jul 4, '01".into(), &"EEE, MMM d, ''yy".into())).to(be_ok());
    expect!(validate_datetime(&"12:08 PM".into(), &"h:mm a".into())).to(be_ok());
//    "hh 'o''clock' a, zzzz"	12 o'clock PM, Pacific Daylight Time
//    "K:mm a, z"	0:08 PM, AEST
    expect!(validate_datetime(&"02001.July.04 AD 12:08 PM".into(), &"yyyyy.MMMMM.dd GGG hh:mm aaa".into())).to(be_ok());
    expect!(validate_datetime(&"Wed, 4 Jul 2001 12:08:56 -0700".into(), &"EEE, d MMM yyyy HH:mm:ss Z".into())).to(be_ok());
    expect!(validate_datetime(&"010704120856-0700".into(), &"yyMMddHHmmssZ".into())).to(be_ok());
    expect!(validate_datetime(&"2001-07-04T12:08:56.235-0700".into(), &"yyyy-MM-dd'T'HH:mm:ss.SSSZ".into())).to(be_ok());
    expect!(validate_datetime(&"2001-07-04T12:08:56.235-07:00".into(), &"yyyy-MM-dd'T'HH:mm:ss.SSSXXX".into())).to(be_ok());
    expect!(validate_datetime(&"2001-W27-3".into(), &"YYYY-'W'ww-u".into())).to(be_ok());
  }

  #[test]
  fn parse_era() {
    expect!(parse_pattern(CompleteStr("G"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Era])));
    expect!(parse_pattern(CompleteStr("GG"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Era])));
    expect!(parse_pattern(CompleteStr("GGGGG"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Era])));

    expect!(validate_datetime(&"ad".into(), &"G".into())).to(be_ok());
    expect!(validate_datetime(&"AD".into(), &"GG".into())).to(be_ok());
    expect!(validate_datetime(&"bc".into(), &"GGG".into())).to(be_ok());
    expect!(validate_datetime(&"BC".into(), &"G".into())).to(be_ok());
    expect!(validate_datetime(&"BX".into(), &"G".into())).to(be_err());
  }

  #[test]
  fn parse_ampm() {
    expect!(parse_pattern(CompleteStr("a"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::AmPm])));
    expect!(parse_pattern(CompleteStr("aa"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::AmPm])));
    expect!(parse_pattern(CompleteStr("aaaa"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::AmPm])));

    expect!(validate_datetime(&"am".into(), &"a".into())).to(be_ok());
    expect!(validate_datetime(&"AM".into(), &"aa".into())).to(be_ok());
    expect!(validate_datetime(&"pm".into(), &"aa".into())).to(be_ok());
    expect!(validate_datetime(&"PM".into(), &"a".into())).to(be_ok());
    expect!(validate_datetime(&"PX".into(), &"a".into())).to(be_err());
  }

  #[test]
  fn parse_year() {
    expect!(parse_pattern(CompleteStr("y"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Year(1)])));
    expect!(parse_pattern(CompleteStr("yy"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Year(2)])));
    expect!(parse_pattern(CompleteStr("yyyy"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Year(4)])));
    expect!(parse_pattern(CompleteStr("YYyy"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Year(4)])));

    expect!(validate_datetime(&"2000".into(), &"yyyy".into())).to(be_ok());
    expect!(validate_datetime(&"20".into(), &"yy".into())).to(be_ok());
    expect!(validate_datetime(&"2000".into(), &"YYYY".into())).to(be_ok());
    expect!(validate_datetime(&"20".into(), &"YY".into())).to(be_ok());
    expect!(validate_datetime(&"20".into(), &"yyyy".into())).to(be_ok());
    expect!(validate_datetime(&"".into(), &"yyyy".into())).to(be_err());
  }

  #[test]
  fn parse_month() {
    expect!(parse_pattern(CompleteStr("M"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Month(1)])));
    expect!(parse_pattern(CompleteStr("MM"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Month(2)])));
    expect!(parse_pattern(CompleteStr("LLL"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Month(3)])));

    expect!(validate_datetime(&"jan".into(), &"M".into())).to(be_ok());
    expect!(validate_datetime(&"october".into(), &"MMM".into())).to(be_ok());
    expect!(validate_datetime(&"December".into(), &"L".into())).to(be_ok());
    expect!(validate_datetime(&"01".into(), &"L".into())).to(be_ok());
    expect!(validate_datetime(&"10".into(), &"MM".into())).to(be_ok());
    expect!(validate_datetime(&"100".into(), &"MM".into())).to(be_err());
    expect!(validate_datetime(&"13".into(), &"MM".into())).to(be_err());
    expect!(validate_datetime(&"31".into(), &"MM".into())).to(be_err());
    expect!(validate_datetime(&"00".into(), &"MM".into())).to(be_err());
    expect!(validate_datetime(&"".into(), &"MMM".into())).to(be_err());
  }

  #[test]
  fn parse_text() {
    expect!(parse_pattern(CompleteStr("ello"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Text("ello".chars().collect())])));
    expect!(parse_pattern(CompleteStr("'dd-MM-yyyy'"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Text("dd-MM-yyyy".chars().collect())])));
    expect!(parse_pattern(CompleteStr("''"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Text("'".chars().collect())])));
    expect!(parse_pattern(CompleteStr("'dd-''MM''-yyyy'"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Text("dd-'MM'-yyyy".chars().collect())])));

    expect!(validate_datetime(&"ello".into(), &"ello".into())).to(be_ok());
    expect!(validate_datetime(&"elo".into(), &"ello".into())).to(be_err());
    expect!(validate_datetime(&"dd-MM-yyyy".into(), &"'dd-MM-yyyy'".into())).to(be_ok());
  }

  #[test]
  fn parse_week_number() {
    expect!(parse_pattern(CompleteStr("wW"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::WeekInYear, DateTimePatternToken::WeekInMonth])));
    expect!(parse_pattern(CompleteStr("www"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::WeekInYear])));
    expect!(parse_pattern(CompleteStr("WW"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::WeekInMonth])));

    expect!(validate_datetime(&"12".into(), &"w".into())).to(be_ok());
    expect!(validate_datetime(&"3".into(), &"WW".into())).to(be_ok());
    expect!(validate_datetime(&"57".into(), &"ww".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"W".into())).to(be_err());
  }

  #[test]
  fn parse_day_number() {
    expect!(parse_pattern(CompleteStr("dD"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::DayInMonth, DateTimePatternToken::DayInYear])));
    expect!(parse_pattern(CompleteStr("dd"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::DayInMonth])));
    expect!(parse_pattern(CompleteStr("DDD"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::DayInYear])));

    expect!(validate_datetime(&"12".into(), &"d".into())).to(be_ok());
    expect!(validate_datetime(&"03".into(), &"DD".into())).to(be_ok());
    expect!(validate_datetime(&"32".into(), &"dd".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"D".into())).to(be_err());
  }

  #[test]
  fn parse_day_of_week() {
    expect!(parse_pattern(CompleteStr("F"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::DayOfWeekInMonth])));
    expect!(parse_pattern(CompleteStr("EE"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::DayName(2)])));
    expect!(parse_pattern(CompleteStr("u"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::DayOfWeek])));

    expect!(validate_datetime(&"12".into(), &"F".into())).to(be_ok());
    expect!(validate_datetime(&"Tue".into(), &"EEE".into())).to(be_ok());
    expect!(validate_datetime(&"Tuesday".into(), &"EEE".into())).to(be_ok());
    expect!(validate_datetime(&"3".into(), &"u".into())).to(be_ok());
    expect!(validate_datetime(&"32".into(), &"u".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"u".into())).to(be_err());
  }

  #[test]
  fn parse_hour() {
    expect!(parse_pattern(CompleteStr("k"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Hour24])));
    expect!(parse_pattern(CompleteStr("KK"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Hour12ZeroBased])));
    expect!(parse_pattern(CompleteStr("hh"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Hour12])));
    expect!(parse_pattern(CompleteStr("HHHH"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Hour24ZeroBased])));

    expect!(validate_datetime(&"11".into(), &"k".into())).to(be_ok());
    expect!(validate_datetime(&"11".into(), &"KK".into())).to(be_ok());
    expect!(validate_datetime(&"11".into(), &"hh".into())).to(be_ok());
    expect!(validate_datetime(&"11".into(), &"H".into())).to(be_ok());

    expect!(validate_datetime(&"25".into(), &"kk".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"k".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"KK".into())).to(be_ok());
    expect!(validate_datetime(&"12".into(), &"KK".into())).to(be_err());
    expect!(validate_datetime(&"12".into(), &"h".into())).to(be_ok());
    expect!(validate_datetime(&"0".into(), &"hh".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"H".into())).to(be_ok());
    expect!(validate_datetime(&"23".into(), &"H".into())).to(be_ok());
    expect!(validate_datetime(&"24".into(), &"HH".into())).to(be_err());
  }

  #[test]
  fn parse_minute_and_second() {
    expect!(parse_pattern(CompleteStr("m"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Minute])));
    expect!(parse_pattern(CompleteStr("s"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Second])));
    expect!(parse_pattern(CompleteStr("SSS"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Millisecond])));

    expect!(validate_datetime(&"12".into(), &"m".into())).to(be_ok());
    expect!(validate_datetime(&"03".into(), &"ss".into())).to(be_ok());
    expect!(validate_datetime(&"030".into(), &"SSS".into())).to(be_ok());
    expect!(validate_datetime(&"60".into(), &"m".into())).to(be_err());
    expect!(validate_datetime(&"60".into(), &"s".into())).to(be_err());
    expect!(validate_datetime(&"1000".into(), &"SS".into())).to(be_err());
  }

  #[test]
  fn parse_timezone() {
    expect!(parse_pattern(CompleteStr("z"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Timezone])));
    expect!(parse_pattern(CompleteStr("Z"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Rfc822Timezone])));
    expect!(parse_pattern(CompleteStr("XXX"))).to(
      be_ok().value((CompleteStr(""), vec![DateTimePatternToken::Iso8601Timezone])));

    expect!(validate_datetime(&"-0700".into(), &"Z".into())).to(be_ok());
    expect!(validate_datetime(&"1100".into(), &"ZZZZ".into())).to(be_err());
    expect!(validate_datetime(&"+1030".into(), &"Z".into())).to(be_ok());
    expect!(validate_datetime(&"-2400".into(), &"Z".into())).to(be_err());
    expect!(validate_datetime(&"2361".into(), &"Z".into())).to(be_err());

    expect!(validate_datetime(&"-0700".into(), &"X".into())).to(be_ok());
    expect!(validate_datetime(&"-08".into(), &"X".into())).to(be_ok());
    expect!(validate_datetime(&"1100".into(), &"XXXX".into())).to(be_err());
    expect!(validate_datetime(&"+1030".into(), &"X".into())).to(be_ok());
    expect!(validate_datetime(&"+10:30".into(), &"X".into())).to(be_ok());
    expect!(validate_datetime(&"Z".into(), &"X".into())).to(be_ok());
    expect!(validate_datetime(&"-2400".into(), &"X".into())).to(be_err());
    expect!(validate_datetime(&"2361".into(), &"X".into())).to(be_err());

    expect!(validate_datetime(&"GMT-0:00".into(), &"z".into())).to(be_ok());
    expect!(validate_datetime(&"UTC-0:00".into(), &"z".into())).to(be_ok());
    expect!(validate_datetime(&"UTC".into(), &"z".into())).to(be_ok());
    expect!(validate_datetime(&"GMT+10:00".into(), &"z".into())).to(be_ok());
    expect!(validate_datetime(&"GMT+10:30".into(), &"z".into())).to(be_ok());
    expect!(validate_datetime(&"1100".into(), &"zzzz".into())).to(be_err());
    expect!(validate_datetime(&"GMT-24:00".into(), &"z".into())).to(be_err());
    expect!(validate_datetime(&"GMT+23:61".into(), &"z".into())).to(be_err());
    expect!(validate_datetime(&"GMT+2351".into(), &"z".into())).to(be_err());
  }

  #[test]
  fn to_chrono_pattern_test() {
    expect!(to_chrono_pattern(&parse_pattern(CompleteStr("yyyy-MM-dd")).unwrap().1)).to(be_equal_to("%Y-%m-%d"));
    expect!(to_chrono_pattern(&parse_pattern(CompleteStr("yyyy-MM-dd HH:mm:ss")).unwrap().1)).to(be_equal_to("%Y-%m-%d %H:%M:%S"));
    expect!(to_chrono_pattern(&parse_pattern(CompleteStr("EEE, MMM d, ''yy")).unwrap().1)).to(be_equal_to("%a, %b %d, \'%y"));
    expect!(to_chrono_pattern(&parse_pattern(CompleteStr("h:mm a")).unwrap().1)).to(be_equal_to("%I:%M %p"));
    expect!(to_chrono_pattern(&parse_pattern(CompleteStr("hh 'o''clock' a, z")).unwrap().1)).to(be_equal_to("%I o'clock %p, %:z"));
    expect!(to_chrono_pattern(&parse_pattern(CompleteStr("yyyyy.MMMMM.dd GGG hh:mm aaa")).unwrap().1)).to(be_equal_to("%Y.%B.%d AD %I:%M %p"));
    expect!(to_chrono_pattern(&parse_pattern(CompleteStr("EEE, d MMM yyyy HH:mm:ss Z")).unwrap().1)).to(be_equal_to("%a, %d %b %Y %H:%M:%S %z"));
    expect!(to_chrono_pattern(&parse_pattern(CompleteStr("yyMMddHHmmssZ")).unwrap().1)).to(be_equal_to("%y%m%d%H%M%S%z"));
    expect!(to_chrono_pattern(&parse_pattern(CompleteStr("yyyy-MM-dd'T'HH:mm:ss.SSSZ")).unwrap().1)).to(be_equal_to("%Y-%m-%dT%H:%M:%S.%3f%z"));
    expect!(to_chrono_pattern(&parse_pattern(CompleteStr("yyyy-MM-dd'T'HH:mm:ss.SSSXXX")).unwrap().1)).to(be_equal_to("%Y-%m-%dT%H:%M:%S.%3f%:z"));
    expect!(to_chrono_pattern(&parse_pattern(CompleteStr("YYYY-'W'ww-u")).unwrap().1)).to(be_equal_to("%Y-W%U-%u"));
  }

}
