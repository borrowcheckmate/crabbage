use std::time::Duration;

/// Reads an ISO 8601 duration such as `PT1H30M`, `P0DT20M` or `PT90S`. Days, hours, minutes and
/// whole seconds only; months, years, fractions and empty durations give `None`.
pub(super) fn parse(text: &str) -> Option<Duration> {
    let body = text.trim().strip_prefix('P')?;
    let mut seconds: u64 = 0;
    let mut number = String::new();
    let mut in_time = false;
    let mut fields = 0_u32;
    for c in body.chars() {
        match c {
            '0'..='9' => number.push(c),
            'T' if !in_time && number.is_empty() => in_time = true,
            _ => {
                let factor = designator(c, in_time)?;
                let value: u64 = number.parse().ok()?;
                seconds = seconds.checked_add(value.checked_mul(factor)?)?;
                number.clear();
                fields += 1;
            }
        }
    }
    (number.is_empty() && fields > 0).then(|| Duration::from_secs(seconds))
}

const fn designator(c: char, in_time: bool) -> Option<u64> {
    match (c, in_time) {
        ('D', false) => Some(86_400),
        ('H', true) => Some(3_600),
        ('M', true) => Some(60),
        ('S', true) => Some(1),
        _ => None,
    }
}
