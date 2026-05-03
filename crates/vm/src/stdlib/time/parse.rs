use crate::{Program, Value, ValueData, Vm, VmError};

pub(crate) const ANSIC_LAYOUT: &str = "Mon Jan _2 15:04:05 2006";
pub(crate) const RFC850_LAYOUT: &str = "Monday, 02-Jan-06 15:04:05 MST";
pub(super) const RFC1123_LAYOUT: &str = "Mon, 02 Jan 2006 15:04:05 MST";
pub(super) const RFC1123Z_LAYOUT: &str = "Mon, 02 Jan 2006 15:04:05 -0700";
pub(super) const RFC3339_LAYOUT: &str = "2006-01-02T15:04:05Z07:00";

const HTTP_TIME_FORMAT_LAYOUT: &str = "Mon, 02 Jan 2006 15:04:05 GMT";
const SECOND: i64 = 1_000_000_000;
const SECONDS_PER_DAY: i64 = 86_400;
const SHORT_WEEKDAYS: &[&str] = &["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const LONG_WEEKDAYS: &[&str] = &[
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const SHORT_MONTHS: &[&str] = &[
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub(crate) fn time_parse(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let layout = string_arg(vm, program, "time.Parse", &args[0])?;
    let value = string_arg(vm, program, "time.Parse", &args[1])?;

    match parse_time_layout(&layout, &value) {
        Ok(unix_nanos) => Ok(vec![super::core::time_value(unix_nanos), Value::nil()]),
        Err(message) => Ok(vec![super::core::time_value(0), Value::error(message)]),
    }
}

fn parse_time_layout(layout: &str, value: &str) -> Result<i64, String> {
    let context = ParseContext { layout, value };
    let parsed = match layout {
        HTTP_TIME_FORMAT_LAYOUT => parse_http_time_format(&context)?,
        RFC1123_LAYOUT => parse_rfc1123(&context)?,
        RFC1123Z_LAYOUT => parse_rfc1123z(&context)?,
        RFC3339_LAYOUT => parse_rfc3339(&context)?,
        RFC850_LAYOUT => parse_rfc850(&context)?,
        ANSIC_LAYOUT => parse_ansic(&context)?,
        _ => return Err(context.unsupported_layout()),
    };

    build_unix_nanos(&context, parsed)
}

#[derive(Clone, Copy)]
struct ParsedDateTime {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
    offset_seconds: i64,
}

struct ParseContext<'a> {
    layout: &'a str,
    value: &'a str,
}

impl ParseContext<'_> {
    fn cannot_parse(&self, value_elem: &str, layout_elem: &str) -> String {
        format!(
            "parsing time {:?} as {:?}: cannot parse {:?} as {:?}",
            self.value, self.layout, value_elem, layout_elem
        )
    }

    fn extra_text(&self, value_elem: &str) -> String {
        format!(
            "parsing time {:?} as {:?}: extra text: {:?}",
            self.value, self.layout, value_elem
        )
    }

    fn message(&self, message: &str) -> String {
        format!("parsing time {:?}: {message}", self.value)
    }

    fn unsupported_layout(&self) -> String {
        format!(
            "parsing time {:?} as {:?}: unsupported layout",
            self.value, self.layout
        )
    }
}

fn parse_http_time_format(context: &ParseContext<'_>) -> Result<ParsedDateTime, String> {
    let mut remaining = context.value;
    remaining = consume_short_weekday(context, remaining)?;
    remaining = consume_prefix(context, remaining, ", ")?;
    let (day, rest) = consume_two_digits(context, remaining, "02")?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (month, rest) = consume_short_month(context, remaining)?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (year, rest) = consume_four_digits(context, remaining, "2006")?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (hour, minute, second, rest) = consume_time_of_day(context, remaining)?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " GMT")?;
    if !remaining.is_empty() {
        return Err(context.extra_text(remaining));
    }
    Ok(ParsedDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
        offset_seconds: 0,
    })
}

fn parse_rfc1123(context: &ParseContext<'_>) -> Result<ParsedDateTime, String> {
    let mut remaining = context.value;
    remaining = consume_short_weekday(context, remaining)?;
    remaining = consume_prefix(context, remaining, ", ")?;
    let (day, rest) = consume_two_digits(context, remaining, "02")?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (month, rest) = consume_short_month(context, remaining)?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (year, rest) = consume_four_digits(context, remaining, "2006")?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (hour, minute, second, rest) = consume_time_of_day(context, remaining)?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let remaining = consume_zone_name(context, remaining)?;
    if !remaining.is_empty() {
        return Err(context.extra_text(remaining));
    }
    Ok(ParsedDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
        offset_seconds: 0,
    })
}

fn parse_rfc1123z(context: &ParseContext<'_>) -> Result<ParsedDateTime, String> {
    let mut remaining = context.value;
    remaining = consume_short_weekday(context, remaining)?;
    remaining = consume_prefix(context, remaining, ", ")?;
    let (day, rest) = consume_two_digits(context, remaining, "02")?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (month, rest) = consume_short_month(context, remaining)?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (year, rest) = consume_four_digits(context, remaining, "2006")?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (hour, minute, second, rest) = consume_time_of_day(context, remaining)?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (offset_seconds, remaining) = consume_numeric_zone(context, remaining)?;
    if !remaining.is_empty() {
        return Err(context.extra_text(remaining));
    }
    Ok(ParsedDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
        offset_seconds,
    })
}

fn parse_rfc3339(context: &ParseContext<'_>) -> Result<ParsedDateTime, String> {
    let mut remaining = context.value;
    let (year, rest) = consume_four_digits(context, remaining, "2006")?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, "-")?;
    let (month, rest) = consume_two_digits(context, remaining, "01")?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, "-")?;
    let (day, rest) = consume_two_digits(context, remaining, "02")?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, "T")?;
    let (hour, minute, second, rest) = consume_time_of_day(context, remaining)?;
    remaining = rest;
    let (offset_seconds, remaining) = consume_rfc3339_zone(context, remaining)?;
    if !remaining.is_empty() {
        return Err(context.extra_text(remaining));
    }
    Ok(ParsedDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
        offset_seconds,
    })
}

fn parse_rfc850(context: &ParseContext<'_>) -> Result<ParsedDateTime, String> {
    let mut remaining = context.value;
    remaining = consume_long_weekday(context, remaining)?;
    remaining = consume_prefix(context, remaining, ", ")?;
    let (day, rest) = consume_two_digits(context, remaining, "02")?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, "-")?;
    let (month, rest) = consume_short_month(context, remaining)?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, "-")?;
    let (year, rest) = consume_two_digit_year(context, remaining)?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (hour, minute, second, rest) = consume_time_of_day(context, remaining)?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let remaining = consume_zone_name(context, remaining)?;
    if !remaining.is_empty() {
        return Err(context.extra_text(remaining));
    }
    Ok(ParsedDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
        offset_seconds: 0,
    })
}

fn parse_ansic(context: &ParseContext<'_>) -> Result<ParsedDateTime, String> {
    let mut remaining = context.value;
    remaining = consume_short_weekday(context, remaining)?;
    remaining = consume_prefix(context, remaining, " ")?;
    let (month, rest) = consume_short_month(context, remaining)?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (day, rest) = consume_under_day(context, remaining)?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (hour, minute, second, rest) = consume_time_of_day(context, remaining)?;
    remaining = rest;
    remaining = consume_prefix(context, remaining, " ")?;
    let (year, remaining) = consume_four_digits(context, remaining, "2006")?;
    if !remaining.is_empty() {
        return Err(context.extra_text(remaining));
    }
    Ok(ParsedDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
        offset_seconds: 0,
    })
}

fn build_unix_nanos(context: &ParseContext<'_>, parsed: ParsedDateTime) -> Result<i64, String> {
    if !(0..=12).contains(&parsed.month) || parsed.month == 0 {
        return Err(context.message("month out of range"));
    }
    if !(0..24).contains(&parsed.hour) {
        return Err(context.message("hour out of range"));
    }
    if !(0..60).contains(&parsed.minute) {
        return Err(context.message("minute out of range"));
    }
    if !(0..60).contains(&parsed.second) {
        return Err(context.message("second out of range"));
    }
    if parsed.day < 1 || parsed.day > days_in_month(parsed.year, parsed.month) {
        return Err(context.message("day out of range"));
    }

    let days = days_from_civil(parsed.year, parsed.month, parsed.day);
    let total_seconds = i128::from(days) * i128::from(SECONDS_PER_DAY)
        + i128::from(parsed.hour) * 3_600
        + i128::from(parsed.minute) * 60
        + i128::from(parsed.second)
        - i128::from(parsed.offset_seconds);
    let unix_nanos = total_seconds * i128::from(SECOND);
    i64::try_from(unix_nanos).map_err(|_| context.message("time out of range"))
}

fn consume_prefix<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
    prefix: &str,
) -> Result<&'a str, String> {
    remaining
        .strip_prefix(prefix)
        .ok_or_else(|| context.cannot_parse(remaining, prefix))
}

fn consume_short_weekday<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
) -> Result<&'a str, String> {
    consume_name(context, remaining, SHORT_WEEKDAYS, "Mon")
}

fn consume_long_weekday<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
) -> Result<&'a str, String> {
    consume_name(context, remaining, LONG_WEEKDAYS, "Monday")
}

fn consume_short_month<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
) -> Result<(i64, &'a str), String> {
    for (index, name) in SHORT_MONTHS.iter().enumerate() {
        if let Some(rest) = remaining.strip_prefix(name) {
            return Ok((i64::try_from(index + 1).unwrap_or(0), rest));
        }
    }
    Err(context.cannot_parse(remaining, "Jan"))
}

fn consume_name<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
    names: &[&str],
    layout_elem: &str,
) -> Result<&'a str, String> {
    for name in names {
        if let Some(rest) = remaining.strip_prefix(name) {
            return Ok(rest);
        }
    }
    Err(context.cannot_parse(remaining, layout_elem))
}

fn consume_two_digits<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
    layout_elem: &str,
) -> Result<(i64, &'a str), String> {
    if remaining.len() < 2
        || !remaining.as_bytes()[0].is_ascii_digit()
        || !remaining.as_bytes()[1].is_ascii_digit()
    {
        return Err(context.cannot_parse(remaining, layout_elem));
    }

    let value =
        i64::from(remaining.as_bytes()[0] - b'0') * 10 + i64::from(remaining.as_bytes()[1] - b'0');
    Ok((value, &remaining[2..]))
}

fn consume_four_digits<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
    layout_elem: &str,
) -> Result<(i64, &'a str), String> {
    if remaining.len() < 4 || !remaining.as_bytes()[..4].iter().all(u8::is_ascii_digit) {
        return Err(context.cannot_parse(remaining, layout_elem));
    }

    let mut value = 0i64;
    for byte in &remaining.as_bytes()[..4] {
        value = value * 10 + i64::from(*byte - b'0');
    }
    Ok((value, &remaining[4..]))
}

fn consume_two_digit_year<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
) -> Result<(i64, &'a str), String> {
    let (year, remaining) = consume_two_digits(context, remaining, "06")?;
    let expanded = if year >= 69 { 1900 + year } else { 2000 + year };
    Ok((expanded, remaining))
}

fn consume_under_day<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
) -> Result<(i64, &'a str), String> {
    let remaining = remaining.strip_prefix(' ').unwrap_or(remaining);
    consume_one_or_two_digits(context, remaining, "_2")
}

fn consume_one_or_two_digits<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
    layout_elem: &str,
) -> Result<(i64, &'a str), String> {
    let Some(first) = remaining.as_bytes().first() else {
        return Err(context.cannot_parse(remaining, layout_elem));
    };
    if !first.is_ascii_digit() {
        return Err(context.cannot_parse(remaining, layout_elem));
    }

    let mut value = i64::from(*first - b'0');
    let mut index = 1usize;
    if remaining.as_bytes().get(1).is_some_and(u8::is_ascii_digit) {
        value = value * 10 + i64::from(remaining.as_bytes()[1] - b'0');
        index = 2;
    }
    Ok((value, &remaining[index..]))
}

fn consume_time_of_day<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
) -> Result<(i64, i64, i64, &'a str), String> {
    let (hour, remaining) = consume_two_digits(context, remaining, "15")?;
    let remaining = consume_prefix(context, remaining, ":")?;
    let (minute, remaining) = consume_two_digits(context, remaining, "04")?;
    let remaining = consume_prefix(context, remaining, ":")?;
    let (second, remaining) = consume_two_digits(context, remaining, "05")?;
    Ok((hour, minute, second, remaining))
}

fn consume_zone_name<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
) -> Result<&'a str, String> {
    let zone_len = remaining
        .bytes()
        .take_while(|byte| byte.is_ascii_alphabetic())
        .count();
    if zone_len < 3 {
        return Err(context.cannot_parse(remaining, "MST"));
    }
    Ok(&remaining[zone_len..])
}

fn consume_numeric_zone<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
) -> Result<(i64, &'a str), String> {
    let Some(sign) = remaining.as_bytes().first() else {
        return Err(context.cannot_parse(remaining, "-0700"));
    };
    if *sign != b'+' && *sign != b'-' {
        return Err(context.cannot_parse(remaining, "-0700"));
    }

    let (hours, remaining) = consume_two_digits(context, &remaining[1..], "07")?;
    let (minutes, remaining) = consume_two_digits(context, remaining, "00")?;
    if hours >= 24 || minutes >= 60 {
        return Err(context.message("time zone offset out of range"));
    }

    let sign = if *sign == b'+' { 1 } else { -1 };
    let offset_seconds = sign * (hours * 3_600 + minutes * 60);
    Ok((offset_seconds, remaining))
}

fn consume_rfc3339_zone<'a>(
    context: &ParseContext<'_>,
    remaining: &'a str,
) -> Result<(i64, &'a str), String> {
    if let Some(remaining) = remaining.strip_prefix('Z') {
        return Ok((0, remaining));
    }

    let Some(sign) = remaining.as_bytes().first() else {
        return Err(context.cannot_parse(remaining, "Z07:00"));
    };
    if *sign != b'+' && *sign != b'-' {
        return Err(context.cannot_parse(remaining, "Z07:00"));
    }

    let (hours, remaining) = consume_two_digits(context, &remaining[1..], "07")?;
    let remaining = consume_prefix(context, remaining, ":")?;
    let (minutes, remaining) = consume_two_digits(context, remaining, "00")?;
    if hours >= 24 || minutes >= 60 {
        return Err(context.message("time zone offset out of range"));
    }

    let sign = if *sign == b'+' { 1 } else { -1 };
    let offset_seconds = sign * (hours * 3_600 + minutes * 60);
    Ok((offset_seconds, remaining))
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let adjusted_year = year - if month <= 2 { 1 } else { 0 };
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    }
    .div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2).div_euclid(5) + day - 1;
    let day_of_era =
        year_of_era * 365 + year_of_era.div_euclid(4) - year_of_era.div_euclid(100) + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn string_arg(vm: &Vm, program: &Program, builtin: &str, value: &Value) -> Result<String, VmError> {
    let ValueData::String(string) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments".into(),
        });
    };
    Ok(string.clone())
}
