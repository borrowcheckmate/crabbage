/// Converts a Gregorian date to days since 1970-01-01.
#[must_use]
pub fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if month == 0 || month > 12 || day == 0 || day > days_in_month(year, month) {
        return None;
    }
    let adjusted_year = year - i32::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = i32::try_from(month).ok()? + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + i32::try_from(day).ok()? - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(i64::from(era * 146_097 + day_of_era - 719_468))
}

/// Converts days since 1970-01-01 to a Gregorian date.
#[must_use]
pub fn civil_from_days(days: i64) -> Option<(i32, u32, u32)> {
    let z = i32::try_from(days.checked_add(719_468)?).ok()?;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i32::from(month <= 2);
    Some((year, u32::try_from(month).ok()?, u32::try_from(day).ok()?))
}

/// Parses an ISO date and returns its day number.
#[must_use]
pub fn parse_iso_date(value: &str) -> Option<i64> {
    let mut parts = value.split('-');
    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let day = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    days_from_civil(year, month, day)
}

/// Formats a day number as an ISO date.
#[must_use]
pub fn format_iso_date(days: i64) -> Option<String> {
    let (year, month, day) = civil_from_days(days)?;
    (year > 0 && year <= 9999).then(|| format!("{year:04}-{month:02}-{day:02}"))
}

const fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use super::{civil_from_days, days_from_civil, format_iso_date, parse_iso_date};

    #[test]
    fn iso_dates_round_trip_and_weekday_arithmetic() {
        let monday = parse_iso_date("2024-02-26").unwrap();
        assert_eq!((monday + 3).rem_euclid(7), 0);
        assert_eq!(format_iso_date(monday + 6).as_deref(), Some("2024-03-03"));
        assert_eq!(civil_from_days(monday), Some((2024, 2, 26)));
        assert_eq!(
            days_from_civil(2024, 2, 29)
                .and_then(format_iso_date)
                .as_deref(),
            Some("2024-02-29")
        );
        assert_eq!(parse_iso_date("2023-02-29"), None);
    }
}
