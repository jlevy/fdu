//! Value grammars for time and size arguments.
//!
//! One parser serves the CLI, the Rust API, and the Python bindings, so every surface
//! accepts exactly the same strings rather than three dialects that drift. `now` is a
//! parameter rather than a call to [`SystemTime::now`] so callers and tests control the
//! reference instant, and so every age in one invocation subtracts from the same moment.
//!
//! The grammars are deliberately closed. Natural-language forms (`yesterday`,
//! `2 weeks ago`) are rejected because they are locale-dependent and unbounded, and
//! calendar units (months, years) are rejected because they require arithmetic that only
//! approximates — a grammar for file ages must not approximate. Every rejection names
//! the spelling that would have worked.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::types::{Error, Result};

/// The most fractional digits an `@epoch` value can carry before the rest is ignored.
const MAX_FRACTION_DIGITS: usize = 9;

/// Parse a `WHEN` value into an absolute instant.
///
/// Accepts `now`, a compound age (`45s`, `2h`, `1h30m`), an RFC 3339 timestamp carrying
/// an offset (`2026-08-10T18:22:31.482919114Z`), or `@` seconds since the Unix epoch
/// (`@1786413716`, `@1786413716.482919114`).
///
/// Ages subtract from `now`, so a caller that captures one instant per invocation gets a
/// consistent window across several arguments.
pub fn parse_when(input: &str, now: SystemTime) -> Result<SystemTime> {
    let value = input.trim();
    if value.is_empty() {
        return Err(when_error(input, "expected `now`, an age like `2h`, or a timestamp"));
    }
    if value.eq_ignore_ascii_case("now") {
        return Ok(now);
    }
    if let Some(epoch) = value.strip_prefix('@') {
        return parse_epoch(input, epoch);
    }
    // Ages are digits and letters only, so a `-` or `:` can only be a timestamp. Deciding
    // by shape rather than by trying each parser keeps the error message specific to what
    // the user was evidently reaching for.
    if value.contains('-') || value.contains(':') {
        return parse_timestamp(input, value);
    }
    let age = parse_age(input, value)?;
    now.checked_sub(age)
        .ok_or_else(|| when_error(input, "age reaches before the representable time range"))
}

/// Parse a `SIZE` value into a byte count.
///
/// Accepts a bare byte count (`512`), decimal suffixes (`10k`, `10KB`, `1.5M`, `2G`), and
/// binary suffixes (`10Ki`, `1.5GiB`). Suffixes are case-insensitive, and a fraction is
/// allowed because `1.5GiB` is how people write sizes.
pub fn parse_size(input: &str) -> Result<u64> {
    let value = input.trim();
    if value.is_empty() {
        return Err(size_error(input, "expected a byte count like `512`, `10M`, or `1.5GiB`"));
    }

    let digits_end = value.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(value.len());
    let (number, suffix) = value.split_at(digits_end);
    if number.is_empty() {
        return Err(size_error(input, "expected a number before the unit, as in `10M`"));
    }

    let factor = size_factor(suffix)
        .ok_or_else(|| size_error(input, &format!("unknown size unit {suffix:?}; use B, K/KB, M/MB, G/GB, T/TB, P/PB, or the binary forms KiB, MiB, GiB, TiB, PiB")))?;

    scale_decimal(number, factor).ok_or_else(|| {
        size_error(input, "size is not a number this machine can represent in bytes")
    })
}

/// Convert an instant to nanoseconds since the Unix epoch, negative before it.
///
/// The index stores timestamps as `i64` nanoseconds, so selection windows have to reach
/// the same representation before they can be compared against entries.
pub fn system_time_to_nanos(time: SystemTime) -> Option<i64> {
    match time.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_nanos()).ok(),
        Err(before) => i64::try_from(before.duration().as_nanos()).ok().map(i64::wrapping_neg),
    }
}

/// Multiply a possibly fractional decimal string by `factor`, in integer arithmetic.
///
/// Floating point would be the obvious implementation and the wrong one: `0.1 * 10^9` is
/// not an integer in binary, and a size filter that is one byte off is a filter that
/// silently drops a file.
fn scale_decimal(number: &str, factor: u64) -> Option<u64> {
    let (whole, fraction) = match number.split_once('.') {
        Some((whole, fraction)) => (whole, fraction),
        None => (number, ""),
    };
    if number.matches('.').count() > 1 || (whole.is_empty() && fraction.is_empty()) {
        return None;
    }

    let factor = u128::from(factor);
    let whole: u128 = if whole.is_empty() { 0 } else { whole.parse().ok()? };
    let mut total = whole.checked_mul(factor)?;

    if !fraction.is_empty() {
        let digits: u128 = fraction.parse().ok()?;
        let scale = 10u128.checked_pow(u32::try_from(fraction.len()).ok()?)?;
        total = total.checked_add(digits.checked_mul(factor)? / scale)?;
    }

    u64::try_from(total).ok()
}

/// Bytes per size suffix, or `None` when the suffix is not part of the grammar.
fn size_factor(suffix: &str) -> Option<u64> {
    const K: u64 = 1_000;
    const KI: u64 = 1_024;
    let unit = suffix.trim().to_ascii_lowercase();
    Some(match unit.as_str() {
        "" | "b" => 1,
        "k" | "kb" => K,
        "m" | "mb" => K.pow(2),
        "g" | "gb" => K.pow(3),
        "t" | "tb" => K.pow(4),
        "p" | "pb" => K.pow(5),
        "ki" | "kib" => KI,
        "mi" | "mib" => KI.pow(2),
        "gi" | "gib" => KI.pow(3),
        "ti" | "tib" => KI.pow(4),
        "pi" | "pib" => KI.pow(5),
        _ => return None,
    })
}

/// Parse a compound age such as `45s`, `2h`, or `1h30m` into a duration.
fn parse_age(input: &str, value: &str) -> Result<Duration> {
    if value.contains(char::is_whitespace) {
        return Err(when_error(
            input,
            "spaces are not part of the age grammar; write compound ages like `1h30m`",
        ));
    }
    if !value.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(when_error(input, "expected `now`, an age like `2h`, or a timestamp"));
    }

    let mut seconds: u64 = 0;
    let mut rest = value;
    while !rest.is_empty() {
        let digits_end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        let (digits, tail) = rest.split_at(digits_end);
        if digits.is_empty() {
            return Err(when_error(input, "expected a number before each unit, as in `1h30m`"));
        }
        if tail.starts_with('.') {
            return Err(when_error(
                input,
                "fractional ages are not supported; write them as compounds, as in `1h30m` rather than `1.5h`",
            ));
        }

        let unit_end = tail.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(tail.len());
        let (unit, remainder) = tail.split_at(unit_end);
        if unit.is_empty() {
            return Err(when_error(
                input,
                "expected a unit after the number: s, m, h, d, or w, as in `45s` or `2h`",
            ));
        }

        let count: u64 = digits
            .parse()
            .map_err(|_| when_error(input, "age is larger than this machine can represent"))?;
        seconds = age_unit_seconds(input, unit)?
            .checked_mul(count)
            .and_then(|scaled| seconds.checked_add(scaled))
            .ok_or_else(|| when_error(input, "age is larger than this machine can represent"))?;
        rest = remainder;
    }

    Ok(Duration::from_secs(seconds))
}

/// Seconds in one age unit, rejecting calendar units with the substitution to use.
fn age_unit_seconds(input: &str, unit: &str) -> Result<u64> {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;
    Ok(match unit.to_ascii_lowercase().as_str() {
        "s" | "sec" | "secs" | "second" | "seconds" => 1,
        "m" | "min" | "mins" | "minute" | "minutes" => MINUTE,
        "h" | "hr" | "hrs" | "hour" | "hours" => HOUR,
        "d" | "day" | "days" => DAY,
        "w" | "week" | "weeks" => 7 * DAY,
        // Calendar units are rejected rather than approximated: a month is not a fixed
        // number of days, and a file age that quietly means 30.44 days is a bug waiting
        // for a bug report nobody can reproduce.
        "mo" | "mon" | "month" | "months" | "y" | "yr" | "yrs" | "year" | "years" => {
            return Err(when_error(
                input,
                "calendar units are not supported because they are not a fixed length; use days, as in `30d` or `365d`",
            ));
        }
        _ => {
            return Err(when_error(
                input,
                &format!("unknown age unit {unit:?}; use s, m, h, d, or w"),
            ));
        }
    })
}

/// Parse `@`-prefixed seconds since the Unix epoch, with an optional fraction.
fn parse_epoch(input: &str, value: &str) -> Result<SystemTime> {
    let (seconds, fraction) = match value.split_once('.') {
        Some((seconds, fraction)) => (seconds, fraction),
        None => (value, ""),
    };
    if seconds.is_empty() && fraction.is_empty() {
        return Err(when_error(input, "expected seconds after `@`, as in `@1786413716`"));
    }

    let negative = seconds.starts_with('-');
    let digits = seconds.strip_prefix(['-', '+']).unwrap_or(seconds);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(when_error(input, "expected a whole number of seconds after `@`"));
    }
    if !fraction.is_empty() && !fraction.bytes().all(|b| b.is_ascii_digit()) {
        return Err(when_error(input, "expected only digits in the fractional seconds after `@`"));
    }

    let seconds: u64 = digits
        .parse()
        .map_err(|_| when_error(input, "epoch seconds are outside the representable range"))?;
    let nanos = fraction_to_nanos(fraction);

    let magnitude = Duration::new(seconds, nanos);
    let instant = if negative {
        UNIX_EPOCH.checked_sub(magnitude)
    } else {
        UNIX_EPOCH.checked_add(magnitude)
    };
    instant.ok_or_else(|| when_error(input, "epoch seconds are outside the representable range"))
}

/// Interpret fractional digits as nanoseconds, padding or truncating to nine.
fn fraction_to_nanos(fraction: &str) -> u32 {
    if fraction.is_empty() {
        return 0;
    }
    let mut nanos: u32 = 0;
    for index in 0..MAX_FRACTION_DIGITS {
        let digit = fraction.as_bytes().get(index).map_or(0, |b| u32::from(b - b'0'));
        nanos = nanos * 10 + digit;
    }
    nanos
}

/// Parse an RFC 3339 timestamp, which must carry its own offset.
///
/// A bare local date or date-time is rejected on purpose: resolving one needs a time-zone
/// database this crate deliberately does not depend on, and guessing UTC would answer a
/// New York prompt with an instant several hours off without saying so.
fn parse_timestamp(input: &str, value: &str) -> Result<SystemTime> {
    let bytes = value.as_bytes();
    if bytes.len() < 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(when_error(
            input,
            "expected a timestamp like `2026-08-10T18:22:31Z`, an age like `2h`, or `now`",
        ));
    }

    let year: i64 = parse_field(input, &value[0..4])?;
    let month: u32 = parse_field(input, &value[5..7])?;
    let day: u32 = parse_field(input, &value[8..10])?;
    let rest = &value[10..];

    if rest.is_empty() || !rest.starts_with(['T', 't', ' ']) {
        return Err(local_time_error(input));
    }
    let time = &rest[1..];

    let offset_index = time
        .find(['Z', 'z', '+'])
        .or_else(|| time.rfind('-'))
        .ok_or_else(|| local_time_error(input))?;
    let (clock, offset) = time.split_at(offset_index);

    if clock.len() < 8 || clock.as_bytes()[2] != b':' || clock.as_bytes()[5] != b':' {
        return Err(when_error(input, "expected a time like `18:22:31` before the offset"));
    }
    let hour: u32 = parse_field(input, &clock[0..2])?;
    let minute: u32 = parse_field(input, &clock[3..5])?;
    let second: u32 = parse_field(input, &clock[6..8])?;
    let nanos = match clock[8..].strip_prefix('.') {
        Some(fraction) if !fraction.is_empty() && fraction.bytes().all(|b| b.is_ascii_digit()) => {
            fraction_to_nanos(fraction)
        }
        Some(_) => return Err(when_error(input, "expected digits after the decimal point")),
        None if clock.len() == 8 => 0,
        None => return Err(when_error(input, "expected `.` before fractional seconds")),
    };

    validate_civil(input, month, day, hour, minute, second, year)?;
    let offset_seconds = parse_offset(input, offset)?;

    let days = days_from_civil(year, month, day);
    let seconds = days
        .checked_mul(86_400)
        .and_then(|day_seconds| day_seconds.checked_add(i64::from(hour) * 3_600))
        .and_then(|partial| partial.checked_add(i64::from(minute) * 60))
        .and_then(|partial| partial.checked_add(i64::from(second)))
        .and_then(|utc| utc.checked_sub(offset_seconds))
        .ok_or_else(|| when_error(input, "timestamp is outside the representable range"))?;

    epoch_offset(seconds, nanos)
        .ok_or_else(|| when_error(input, "timestamp is outside the representable range"))
}

/// Build an instant from signed epoch seconds plus a nanosecond remainder.
fn epoch_offset(seconds: i64, nanos: u32) -> Option<SystemTime> {
    if seconds >= 0 {
        return UNIX_EPOCH.checked_add(Duration::new(u64::try_from(seconds).ok()?, nanos));
    }
    // Nanoseconds always run forward from the second, so a negative second with a
    // fraction is one second further back plus the fraction forward.
    let magnitude = u64::try_from(seconds.checked_neg()?).ok()?;
    let before = UNIX_EPOCH.checked_sub(Duration::from_secs(magnitude))?;
    before.checked_add(Duration::from_nanos(u64::from(nanos)))
}

/// Parse a fixed-width numeric field, rejecting signs and stray characters.
fn parse_field<T: std::str::FromStr>(input: &str, field: &str) -> Result<T> {
    if field.is_empty() || !field.bytes().all(|b| b.is_ascii_digit()) {
        return Err(when_error(input, "expected digits in every timestamp field"));
    }
    field.parse().map_err(|_| when_error(input, "timestamp field is out of range"))
}

/// Reject civil fields that no calendar produces.
fn validate_civil(
    input: &str,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    year: i64,
) -> Result<()> {
    if !(1..=12).contains(&month) {
        return Err(when_error(input, "month must be between 01 and 12"));
    }
    if day < 1 || day > days_in_month(year, month) {
        return Err(when_error(input, "day is not a day of that month"));
    }
    if hour > 23 {
        return Err(when_error(input, "hour must be between 00 and 23"));
    }
    if minute > 59 {
        return Err(when_error(input, "minute must be between 00 and 59"));
    }
    // RFC 3339 allows `60` for a leap second; treating it as the following instant keeps
    // a legal timestamp parseable without pretending we track leap seconds.
    if second > 60 {
        return Err(when_error(input, "second must be between 00 and 60"));
    }
    Ok(())
}

/// Days in a month, accounting for leap years.
fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Whether a proleptic Gregorian year is a leap year.
fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Parse `Z` or `±HH:MM` into seconds east of UTC.
fn parse_offset(input: &str, offset: &str) -> Result<i64> {
    if offset.eq_ignore_ascii_case("z") {
        return Ok(0);
    }
    let (sign, rest) = match offset.split_at(1) {
        ("+", rest) => (1, rest),
        ("-", rest) => (-1, rest),
        _ => return Err(local_time_error(input)),
    };
    if rest.len() != 5 || rest.as_bytes()[2] != b':' {
        return Err(when_error(input, "expected an offset like `Z`, `+05:30`, or `-08:00`"));
    }
    let hours: i64 = parse_field(input, &rest[0..2])?;
    let minutes: i64 = parse_field(input, &rest[3..5])?;
    if hours > 23 || minutes > 59 {
        return Err(when_error(input, "offset must be within ±23:59"));
    }
    Ok(sign * (hours * 3_600 + minutes * 60))
}

/// Days from the Unix epoch to a proleptic Gregorian date.
///
/// Howard Hinnant's `days_from_civil`, which is exact for every year this crate can
/// represent and needs no lookup table.
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = i64::from(month);
    let day_of_year =
        (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The rejection for a timestamp with no offset, which is the local-time form.
fn local_time_error(input: &str) -> Error {
    when_error(
        input,
        "local date and time are not supported yet because resolving one needs a time-zone database; write an RFC 3339 timestamp with an offset, as in `2026-08-10T12:30:00Z` or `2026-08-10T12:30:00-08:00`, or use `@` epoch seconds",
    )
}

/// Build a time-grammar rejection.
fn when_error(input: &str, hint: &str) -> Error {
    Error::InvalidValue { kind: "time", value: input.to_string(), hint: hint.to_string() }
}

/// Build a size-grammar rejection.
fn size_error(input: &str, hint: &str) -> Error {
    Error::InvalidValue { kind: "size", value: input.to_string(), hint: hint.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed reference instant so every age assertion is exact: 2026-08-10T18:22:31Z.
    const NOW_SECS: u64 = 1_786_386_151;

    fn now() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(NOW_SECS)
    }

    fn seconds_before_now(value: &str) -> u64 {
        let parsed = parse_when(value, now()).expect("value parses");
        now().duration_since(parsed).expect("not in the future").as_secs()
    }

    fn epoch_nanos(value: &str) -> i64 {
        let parsed = parse_when(value, now()).expect("value parses");
        system_time_to_nanos(parsed).expect("representable")
    }

    fn time_rejection(value: &str) -> String {
        match parse_when(value, now()) {
            Err(Error::InvalidValue { kind: "time", hint, .. }) => hint,
            other => panic!("expected {value:?} to be rejected as a time, got {other:?}"),
        }
    }

    fn size_rejection(value: &str) -> String {
        match parse_size(value) {
            Err(Error::InvalidValue { kind: "size", hint, .. }) => hint,
            other => panic!("expected {value:?} to be rejected as a size, got {other:?}"),
        }
    }

    #[test]
    fn now_keyword_returns_the_reference_instant() {
        assert_eq!(parse_when("now", now()).expect("parses"), now());
        assert_eq!(parse_when("  NOW  ", now()).expect("parses"), now());
    }

    #[test]
    fn ages_subtract_from_the_reference_instant() {
        for (value, expected) in [
            ("45s", 45),
            ("45sec", 45),
            ("45seconds", 45),
            ("2m", 120),
            ("2min", 120),
            ("2minutes", 120),
            ("2h", 7_200),
            ("2hr", 7_200),
            ("2hours", 7_200),
            ("7d", 604_800),
            ("7days", 604_800),
            ("1w", 604_800),
            ("2weeks", 1_209_600),
        ] {
            assert_eq!(seconds_before_now(value), expected, "{value}");
        }
    }

    #[test]
    fn compound_ages_sum_their_parts() {
        assert_eq!(seconds_before_now("1h30m"), 5_400);
        assert_eq!(seconds_before_now("1d12h30m15s"), 131_415);
        // Order is not enforced, because the sum is what the value means.
        assert_eq!(seconds_before_now("30m1h"), 5_400);
    }

    #[test]
    fn rfc3339_timestamps_are_exact_and_honor_their_offset() {
        // The same instant written three ways must parse to the same nanosecond.
        let utc = epoch_nanos("2026-08-10T18:22:31Z");
        assert_eq!(utc, 1_786_386_151_000_000_000);
        assert_eq!(epoch_nanos("2026-08-10T10:22:31-08:00"), utc);
        assert_eq!(epoch_nanos("2026-08-10T23:52:31+05:30"), utc);
        // Lowercase separators are legal RFC 3339.
        assert_eq!(epoch_nanos("2026-08-10t18:22:31z"), utc);
    }

    #[test]
    fn rfc3339_fractions_round_trip_to_the_nanosecond() {
        assert_eq!(
            epoch_nanos("2026-08-10T18:22:31.482919114Z"),
            1_786_386_151_482_919_114,
            "a report's scan_started_at must survive being fed back as a watermark"
        );
        // Shorter fractions pad rather than truncate.
        assert_eq!(epoch_nanos("2026-08-10T18:22:31.5Z"), 1_786_386_151_500_000_000);
    }

    #[test]
    fn epoch_values_accept_an_optional_fraction() {
        assert_eq!(epoch_nanos("@1786386151"), 1_786_386_151_000_000_000);
        assert_eq!(epoch_nanos("@1786386151.482919114"), 1_786_386_151_482_919_114);
        assert_eq!(epoch_nanos("@0"), 0);
        assert_eq!(epoch_nanos("@-1"), -1_000_000_000);
    }

    #[test]
    fn pre_epoch_timestamps_are_negative_nanoseconds() {
        assert_eq!(epoch_nanos("1969-12-31T23:59:59Z"), -1_000_000_000);
        assert_eq!(epoch_nanos("1970-01-01T00:00:00Z"), 0);
    }

    #[test]
    fn leap_days_parse_only_in_leap_years() {
        assert!(parse_when("2024-02-29T00:00:00Z", now()).is_ok());
        assert_eq!(time_rejection("2026-02-29T00:00:00Z"), "day is not a day of that month");
    }

    #[test]
    fn calendar_units_are_rejected_with_a_days_suggestion() {
        for value in ["3mo", "3months", "1y", "2years"] {
            assert!(
                time_rejection(value).contains("use days"),
                "{value} should suggest days, got {:?}",
                time_rejection(value)
            );
        }
    }

    #[test]
    fn fractional_ages_are_rejected_with_a_compound_suggestion() {
        let hint = time_rejection("1.5h");
        assert!(hint.contains("1h30m"), "expected a compound suggestion, got {hint:?}");
    }

    #[test]
    fn natural_language_is_rejected() {
        assert!(time_rejection("yesterday").contains("expected `now`"));
        assert!(time_rejection("2 weeks ago").contains("spaces are not part"));
    }

    #[test]
    fn local_timestamps_are_rejected_rather_than_assumed_to_be_utc() {
        // Guessing UTC here would answer a New York prompt several hours off in silence,
        // which is exactly the failure the freshness rules exist to prevent.
        for value in ["2026-08-10", "2026-08-10 12:30", "2026-08-10T12:30:00"] {
            let hint = time_rejection(value);
            assert!(hint.contains("offset"), "{value} should ask for an offset, got {hint:?}");
        }
    }

    #[test]
    fn malformed_times_name_the_grammar() {
        assert!(time_rejection("").contains("expected `now`"));
        assert!(time_rejection("2h30").contains("expected a unit"));
        assert!(time_rejection("5x").contains("unknown age unit"));
        assert!(time_rejection("2026-13-01T00:00:00Z").contains("month must be"));
        assert!(time_rejection("2026-08-10T25:00:00Z").contains("hour must be"));
    }

    #[test]
    fn sizes_accept_decimal_and_binary_units() {
        for (value, expected) in [
            ("512", 512),
            ("512B", 512),
            ("10k", 10_000),
            ("10KB", 10_000),
            ("10M", 10_000_000),
            ("2G", 2_000_000_000),
            ("1T", 1_000_000_000_000),
            ("10Ki", 10_240),
            ("10KiB", 10_240),
            ("1Mi", 1_048_576),
            ("1GiB", 1_073_741_824),
        ] {
            assert_eq!(parse_size(value).expect("parses"), expected, "{value}");
        }
    }

    #[test]
    fn fractional_sizes_scale_exactly() {
        // Integer arithmetic, not floating point: 1.5 GiB is exactly 1610612736 bytes.
        assert_eq!(parse_size("1.5GiB").expect("parses"), 1_610_612_736);
        assert_eq!(parse_size("1.5M").expect("parses"), 1_500_000);
        assert_eq!(parse_size("0.5k").expect("parses"), 500);
        assert_eq!(parse_size("2.25M").expect("parses"), 2_250_000);
    }

    #[test]
    fn size_units_are_case_insensitive() {
        assert_eq!(parse_size("10mb").expect("parses"), parse_size("10MB").expect("parses"));
        assert_eq!(parse_size("1gib").expect("parses"), parse_size("1GiB").expect("parses"));
    }

    #[test]
    fn malformed_sizes_name_the_units() {
        assert!(size_rejection("").contains("expected a byte count"));
        assert!(size_rejection("M").contains("expected a number"));
        assert!(size_rejection("10X").contains("unknown size unit"));
        assert!(size_rejection("1.2.3M").contains("not a number"));
        // A size that cannot fit in a byte count is rejected, never wrapped.
        assert!(size_rejection("99999999P").contains("not a number"));
    }

    #[test]
    fn nanosecond_conversion_survives_both_sides_of_the_epoch() {
        assert_eq!(system_time_to_nanos(UNIX_EPOCH), Some(0));
        assert_eq!(system_time_to_nanos(UNIX_EPOCH + Duration::from_nanos(5)), Some(5));
        assert_eq!(system_time_to_nanos(UNIX_EPOCH - Duration::from_nanos(5)), Some(-5));
    }
}
