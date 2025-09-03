use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryToken {
    Colon,
    Equal,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Not,
    Or,
    StrLit(String),
    Ident(String),
}

pub struct QueryLexer {
    input: Rc<[char]>,
    read_position: usize,
}
impl QueryLexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            read_position: 0,
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.read_position).copied()
    }

    fn read_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.read_position += 1;
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.read_position += 1;
            } else {
                break;
            }
        }
    }

    fn read_while<F>(&mut self, condition: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while let Some(ch) = self.peek_char() {
            if condition(ch) {
                result.push(ch);
                self.read_position += 1;
            } else {
                break;
            }
        }
        result
    }

    pub fn next_token(&mut self) -> Option<QueryToken> {
        self.skip_whitespace();
        let ch = self.read_char()?;

        let token = match ch {
            ':' => QueryToken::Colon,
            '=' => QueryToken::Equal,
            '<' => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    QueryToken::LessThanOrEqual
                } else {
                    QueryToken::LessThan
                }
            }
            '>' => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    QueryToken::GreaterThanOrEqual
                } else {
                    QueryToken::GreaterThan
                }
            }
            '!' => QueryToken::Not,
            '|' => QueryToken::Or,
            '"' => {
                // Read until the next quote, no escape characters exist
                let str_lit = self.read_while(|c| c != '"');
                self.read_char(); // consume closing quote
                QueryToken::StrLit(str_lit)
            }
            _ => {
                // Not a special character nor whitespace, must be identifier
                let mut ident = String::new();
                ident.push(ch);
                // Read until whitespace or colon. We also allow special characters in identifiers as long as are not at the start
                ident.push_str(&self.read_while(|c| !c.is_whitespace() && c != ':'));
                QueryToken::Ident(ident)
            }
        };
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let input = r#"size:>1000 file:"example.txt" !ext:tmp"#;
        let mut lexer = QueryLexer::new(input);

        let expected_tokens = vec![
            QueryToken::Ident("size".into()),
            QueryToken::Colon,
            QueryToken::GreaterThan,
            QueryToken::Ident("1000".into()),
            QueryToken::Ident("file".into()),
            QueryToken::Colon,
            QueryToken::StrLit("example.txt".into()),
            QueryToken::Not,
            QueryToken::Ident("ext".into()),
            QueryToken::Colon,
            QueryToken::Ident("tmp".into()),
        ];
        for expected in expected_tokens {
            let token = lexer.next_token();
            assert_eq!(token, Some(expected));
        }
    }
    #[test]
    fn test_lexer_with_whitespace() {
        let input = r#"  size :  <=  2048   case : "test file.txt"  "#;
        let mut lexer = QueryLexer::new(input);

        let expected_tokens = vec![
            QueryToken::Ident("size".into()),
            QueryToken::Colon,
            QueryToken::LessThanOrEqual,
            QueryToken::Ident("2048".into()),
            QueryToken::Ident("case".into()),
            QueryToken::Colon,
            QueryToken::StrLit("test file.txt".into()),
        ];
        for expected in expected_tokens {
            let token = lexer.next_token();
            assert_eq!(token, Some(expected));
        }
    }

    #[test]
    fn test_lexer_empty_input() {
        let input = r#"   "#;
        let mut lexer = QueryLexer::new(input);
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lexer_special_characters_in_identifiers() {
        let input = r#"wholefilename:report=v<2.0>!.txt size:>=5000"#;
        let mut lexer = QueryLexer::new(input);
        let expected_tokens = vec![
            QueryToken::Ident("wholefilename".into()),
            QueryToken::Colon,
            QueryToken::Ident("report=v<2.0>!.txt".into()),
            QueryToken::Ident("size".into()),
            QueryToken::Colon,
            QueryToken::GreaterThanOrEqual,
            QueryToken::Ident("5000".into()),
        ];
        for expected in expected_tokens {
            let token = lexer.next_token();
            assert_eq!(token, Some(expected));
        }
    }
    #[test]
    fn test_lexer_unterminated_string() {
        let input = r#"file:"incomplete.txt size:>1000"#;
        let mut lexer = QueryLexer::new(input);

        let expected_tokens = vec![
            QueryToken::Ident("file".into()),
            QueryToken::Colon,
            QueryToken::StrLit("incomplete.txt size:>1000".into()), // Reads until end of input
        ];
        for expected in expected_tokens {
            let token = lexer.next_token();
            assert_eq!(token, Some(expected));
        }
    }
    #[test]
    fn test_lexer_groups() {
        let input = r#"notes.txt < path:homework | size:>100KB >"#;
        let mut lexer = QueryLexer::new(input);
        let expected_tokens = vec![
            QueryToken::Ident("notes.txt".into()),
            QueryToken::LessThan,
            QueryToken::Ident("path".into()),
            QueryToken::Colon,
            QueryToken::Ident("homework".into()),
            QueryToken::Or,
            QueryToken::Ident("size".into()),
            QueryToken::Colon,
            QueryToken::GreaterThan,
            QueryToken::Ident("100KB".into()),
            QueryToken::GreaterThan,
        ];

        for expected in expected_tokens {
            let token = lexer.next_token();
            assert_eq!(token, Some(expected));
        }
    }
}
