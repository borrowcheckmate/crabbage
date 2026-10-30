const ALLOWED_PREFIXES: [&str; 3] = ["///", "//!", "// NOTE:"];

pub(crate) fn plain_comment_lines(source: &str) -> Vec<usize> {
    let mut scanner = Scanner::new(source);
    let mut lines = Vec::new();
    while let Some(current) = scanner.peek(0) {
        match (current, scanner.peek(1)) {
            ('/', Some('/')) => {
                let line = scanner.line;
                if !is_allowed(&scanner.take_line()) {
                    lines.push(line);
                }
            }
            ('/', Some('*')) => {
                lines.push(scanner.line);
                scanner.skip_block_comment();
            }
            ('"', _) => scanner.skip_string(),
            ('r', Some('"' | '#')) if scanner.starts_raw_string() => scanner.skip_raw_string(),
            ('\'', _) => scanner.skip_char_or_lifetime(),
            _ => scanner.advance(),
        }
    }
    lines
}

fn is_allowed(comment: &str) -> bool {
    !comment.starts_with("////") && ALLOWED_PREFIXES.iter().any(|p| comment.starts_with(p))
}

const fn is_identifier(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

struct Scanner {
    chars: Vec<char>,
    pos: usize,
    line: usize,
}

impl Scanner {
    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn behind(&self, offset: usize) -> Option<char> {
        self.pos
            .checked_sub(offset)
            .and_then(|index| self.chars.get(index))
            .copied()
    }

    fn advance(&mut self) {
        if self.peek(0) == Some('\n') {
            self.line += 1;
        }
        self.pos += 1;
    }

    fn advance_by(&mut self, count: usize) {
        for _ in 0..count {
            self.advance();
        }
    }

    fn take_line(&mut self) -> String {
        let mut text = String::new();
        while let Some(c) = self.peek(0).filter(|&c| c != '\n') {
            text.push(c);
            self.advance();
        }
        text
    }

    fn skip_block_comment(&mut self) {
        self.advance_by(2);
        let mut depth = 1_usize;
        while depth > 0 {
            match (self.peek(0), self.peek(1)) {
                (None, _) => return,
                (Some('/'), Some('*')) => {
                    depth += 1;
                    self.advance_by(2);
                }
                (Some('*'), Some('/')) => {
                    depth -= 1;
                    self.advance_by(2);
                }
                _ => self.advance(),
            }
        }
    }

    fn skip_string(&mut self) {
        self.advance();
        while let Some(c) = self.peek(0) {
            self.advance();
            match c {
                '\\' => self.advance(),
                '"' => return,
                _ => {}
            }
        }
    }

    fn starts_raw_string(&self) -> bool {
        let glued_to_identifier = match self.behind(1) {
            Some('b' | 'c') => self.behind(2).is_some_and(is_identifier),
            Some(c) => is_identifier(c),
            None => false,
        };
        if glued_to_identifier {
            return false;
        }
        let mut offset = 1;
        while self.peek(offset) == Some('#') {
            offset += 1;
        }
        self.peek(offset) == Some('"')
    }

    fn skip_raw_string(&mut self) {
        self.advance();
        let mut hashes = 0;
        while self.peek(0) == Some('#') {
            hashes += 1;
            self.advance();
        }
        self.advance();
        while let Some(c) = self.peek(0) {
            if c == '"' && (1..=hashes).all(|offset| self.peek(offset) == Some('#')) {
                self.advance_by(hashes + 1);
                return;
            }
            self.advance();
        }
    }

    fn skip_char_or_lifetime(&mut self) {
        match (self.peek(1), self.peek(2)) {
            (Some('\\'), _) => {
                self.advance_by(3);
                while let Some(c) = self.peek(0) {
                    self.advance();
                    if c == '\'' {
                        return;
                    }
                }
            }
            (Some(_), Some('\'')) => self.advance_by(3),
            _ => self.advance(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::plain_comment_lines;

    #[test]
    fn allows_doc_comments_and_notes() {
        let source = "//! crate\n/// item\nfn a() {}\n// NOTE: required by the ABI\n";
        assert_eq!(plain_comment_lines(source), Vec::<usize>::new());
    }

    #[test]
    fn flags_line_and_trailing_comments() {
        let source = "// section\nfn a() {} // trailing\n//// banner\n";
        assert_eq!(plain_comment_lines(source), [1, 2, 3]);
    }

    #[test]
    fn flags_nested_block_comments_once() {
        let source = "/* outer /* inner */ still */\nfn a() {}\n/** doc block */\n";
        assert_eq!(plain_comment_lines(source), [1, 3]);
    }

    #[test]
    fn ignores_slashes_inside_strings() {
        let source = "let url = \"https://example.com\";\nlet esc = \"\\\" // no\";\n";
        assert_eq!(plain_comment_lines(source), Vec::<usize>::new());
    }

    #[test]
    fn ignores_slashes_inside_raw_strings() {
        let source = "let a = r#\"// \"quoted\" \"#;\nlet b = br\"//\";\n";
        assert_eq!(plain_comment_lines(source), Vec::<usize>::new());
    }

    #[test]
    fn identifiers_ending_in_r_are_not_raw_strings() {
        let source = "let s = tr\"x\";\n// real\n";
        assert_eq!(plain_comment_lines(source), [2]);
    }

    #[test]
    fn distinguishes_chars_from_lifetimes() {
        let source =
            "fn a<'a>(x: &'a str) -> char { '\"' }\nlet q = '\\'';\nlet s = '/';\n// end\n";
        assert_eq!(plain_comment_lines(source), [4]);
    }

    #[test]
    fn counts_lines_inside_multiline_strings() {
        let source = "let s = \"one\ntwo\";\n// here\n";
        assert_eq!(plain_comment_lines(source), [3]);
    }

    #[test]
    fn tolerates_unterminated_input() {
        assert_eq!(plain_comment_lines("/* open"), [1]);
        assert_eq!(plain_comment_lines("\"open"), Vec::<usize>::new());
        assert_eq!(plain_comment_lines("r#\"open"), Vec::<usize>::new());
        assert_eq!(plain_comment_lines("'\\u{1F600"), Vec::<usize>::new());
    }
}
