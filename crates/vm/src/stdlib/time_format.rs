use crate::{Program, Value, ValueData, Vm, VmError};

pub(super) const DATE_TIME_LAYOUT: &str = "2006-01-02 15:04:05";

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

pub(super) fn time_time_format(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let unix_nanos =
        super::time_impl::extract_time_unix_nanos(vm, program, "time.Time.Format", &args[0])?;
    let layout = string_arg(vm, program, "time.Time.Format", &args[1])?;
    Ok(Value::string(format_time_layout_utc(unix_nanos, &layout)))
}

// The current runtime only stores Unix nanoseconds, so formatting projects that
// instant through a UTC-only layout subset with fixed UTC zone renderings.
fn format_time_layout_utc(unix_nanos: i64, layout: &str) -> String {
    let total_seconds = unix_nanos.div_euclid(SECOND);
    let days = total_seconds.div_euclid(SECONDS_PER_DAY);
    let seconds_of_day = total_seconds.rem_euclid(SECONDS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day.div_euclid(3_600);
    let minute = seconds_of_day.rem_euclid(3_600).div_euclid(60);
    let second = seconds_of_day.rem_euclid(60);
    let weekday = SHORT_WEEKDAYS[usize::try_from((days + 4).rem_euclid(7)).unwrap_or(0)];
    let long_weekday = LONG_WEEKDAYS[usize::try_from((days + 4).rem_euclid(7)).unwrap_or(0)];
    let month_name = SHORT_MONTHS[usize::try_from(month - 1).unwrap_or(0)];

    let mut formatted = String::with_capacity(layout.len() + 8);
    let mut index = 0;
    while index < layout.len() {
        let remaining = &layout[index..];
        if remaining.starts_with("Monday") {
            formatted.push_str(long_weekday);
            index += 6;
        } else if remaining.starts_with("Mon") {
            formatted.push_str(weekday);
            index += 3;
        } else if remaining.starts_with("Jan") {
            formatted.push_str(month_name);
            index += 3;
        } else if remaining.starts_with("2006") {
            formatted.push_str(&format!("{year:04}"));
            index += 4;
        } else if remaining.starts_with("-0700") {
            formatted.push_str("+0000");
            index += 5;
        } else if remaining.starts_with("Z07:00") {
            formatted.push('Z');
            index += 6;
        } else if remaining.starts_with("MST") {
            formatted.push_str("UTC");
            index += 3;
        } else if remaining.starts_with("_2") {
            append_space_padded_day(&mut formatted, day);
            index += 2;
        } else if remaining.starts_with("06") {
            append_two_digit(&mut formatted, year.rem_euclid(100));
            index += 2;
        } else if remaining.starts_with("01") {
            append_two_digit(&mut formatted, month);
            index += 2;
        } else if remaining.starts_with("02") {
            append_two_digit(&mut formatted, day);
            index += 2;
        } else if remaining.starts_with("15") {
            append_two_digit(&mut formatted, hour);
            index += 2;
        } else if remaining.starts_with("04") {
            append_two_digit(&mut formatted, minute);
            index += 2;
        } else if remaining.starts_with("05") {
            append_two_digit(&mut formatted, second);
            index += 2;
        } else {
            let ch = remaining.chars().next().unwrap_or_default();
            formatted.push(ch);
            index += ch.len_utf8();
        }
    }

    formatted
}

fn append_two_digit(output: &mut String, value: i64) {
    let tens = u8::try_from(value / 10).unwrap_or(0);
    let ones = u8::try_from(value % 10).unwrap_or(0);
    output.push(char::from(b'0' + tens));
    output.push(char::from(b'0' + ones));
}

fn append_space_padded_day(output: &mut String, day: i64) {
    if day < 10 {
        output.push(' ');
        output.push(char::from(b'0' + u8::try_from(day).unwrap_or(0)));
        return;
    }

    append_two_digit(output, day);
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, i64, i64) {
    let shifted_days = days_since_unix_epoch + 719_468;
    let era = if shifted_days >= 0 {
        shifted_days
    } else {
        shifted_days - 146_096
    }
    .div_euclid(146_097);
    let day_of_era = shifted_days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524
        - day_of_era / 146_096)
        .div_euclid(365);
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2).div_euclid(153);
    let day = day_of_year - (153 * month_prime + 2).div_euclid(5) + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
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
