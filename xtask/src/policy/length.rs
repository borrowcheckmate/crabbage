pub(crate) const MAX_LINES: usize = 500;

pub(crate) fn excess_lines(source: &str) -> Option<usize> {
    let lines = source.lines().count();
    (lines > MAX_LINES).then_some(lines)
}

#[cfg(test)]
mod tests {
    use super::{MAX_LINES, excess_lines};

    #[test]
    fn accepts_the_limit() {
        assert_eq!(excess_lines(&"x\n".repeat(MAX_LINES)), None);
    }

    #[test]
    fn reports_files_over_the_limit() {
        assert_eq!(
            excess_lines(&"x\n".repeat(MAX_LINES + 1)),
            Some(MAX_LINES + 1)
        );
    }
}
