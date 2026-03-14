use ratatui::prelude::*;

/// SQL keywords that should be highlighted
const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "AND", "OR", "NOT", "IN", "IS", "NULL",
    "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE", "CREATE",
    "TABLE", "ALTER", "DROP", "INDEX", "VIEW", "AS", "ON", "JOIN",
    "LEFT", "RIGHT", "INNER", "OUTER", "FULL", "CROSS", "NATURAL",
    "ORDER", "BY", "ASC", "DESC", "GROUP", "HAVING", "LIMIT", "OFFSET",
    "UNION", "ALL", "DISTINCT", "EXISTS", "BETWEEN", "LIKE", "CASE",
    "WHEN", "THEN", "ELSE", "END", "WITH", "RECURSIVE", "EXPLAIN",
    "ANALYZE", "BEGIN", "COMMIT", "ROLLBACK", "TRUNCATE", "GRANT",
    "REVOKE", "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT",
    "DEFAULT", "CHECK", "UNIQUE", "CASCADE", "RETURNING", "PRAGMA",
    "IF", "REPLACE", "MERGE", "USING", "EXCEPT", "INTERSECT",
];

/// SQL aggregate and common functions
const SQL_FUNCTIONS: &[&str] = &[
    "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE", "NULLIF",
    "CAST", "CONVERT", "CONCAT", "LENGTH", "SUBSTR", "SUBSTRING",
    "TRIM", "UPPER", "LOWER", "NOW", "DATE", "EXTRACT", "ROUND",
    "ABS", "CEILING", "FLOOR", "RANDOM", "ROW_NUMBER", "RANK",
    "DENSE_RANK", "LAG", "LEAD", "FIRST_VALUE", "LAST_VALUE",
    "OVER", "PARTITION", "ARRAY_AGG", "STRING_AGG", "JSON_AGG",
];

/// SQL data types
const SQL_TYPES: &[&str] = &[
    "INT", "INTEGER", "BIGINT", "SMALLINT", "TINYINT", "SERIAL",
    "FLOAT", "DOUBLE", "DECIMAL", "NUMERIC", "REAL",
    "VARCHAR", "CHAR", "TEXT", "BLOB", "BOOLEAN", "BOOL",
    "DATE", "TIME", "TIMESTAMP", "INTERVAL", "UUID", "JSON", "JSONB",
];

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Keyword,
    Function,
    DataType,
    StringLiteral,
    Number,
    Comment,
    Operator,
    Identifier,
    Whitespace,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    text: String,
}

/// Tokenize a SQL string into classified tokens for syntax highlighting.
fn tokenize(sql: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = sql.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Whitespace
        if ch.is_whitespace() {
            let start = i;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Whitespace,
                text: chars[start..i].iter().collect(),
            });
            continue;
        }

        // Single-line comment: --
        if ch == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
            let start = i;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Comment,
                text: chars[start..i].iter().collect(),
            });
            continue;
        }

        // String literal: 'text'
        if ch == '\'' {
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\'' {
                    if i + 1 < chars.len() && chars[i + 1] == '\'' {
                        i += 2; // escaped quote
                    } else {
                        i += 1;
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            tokens.push(Token {
                kind: TokenKind::StringLiteral,
                text: chars[start..i].iter().collect(),
            });
            continue;
        }

        // Number
        if ch.is_ascii_digit() || (ch == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Number,
                text: chars[start..i].iter().collect(),
            });
            continue;
        }

        // Operators and punctuation
        if "(),;*+=<>!.".contains(ch) {
            tokens.push(Token {
                kind: TokenKind::Operator,
                text: ch.to_string(),
            });
            i += 1;
            continue;
        }

        // Word (keyword, function, type, or identifier)
        if ch.is_alphabetic() || ch == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let upper = word.to_uppercase();

            let kind = if SQL_KEYWORDS.contains(&upper.as_str()) {
                TokenKind::Keyword
            } else if SQL_FUNCTIONS.contains(&upper.as_str()) {
                TokenKind::Function
            } else if SQL_TYPES.contains(&upper.as_str()) {
                TokenKind::DataType
            } else {
                TokenKind::Identifier
            };

            tokens.push(Token { kind, text: word });
            continue;
        }

        // Anything else
        tokens.push(Token {
            kind: TokenKind::Operator,
            text: ch.to_string(),
        });
        i += 1;
    }

    tokens
}

/// Convert a SQL string into colored ratatui Spans for rendering.
pub fn highlight_sql(sql: &str) -> Line<'static> {
    let tokens = tokenize(sql);
    let spans: Vec<Span<'static>> = tokens
        .into_iter()
        .map(|token| {
            let style = match token.kind {
                TokenKind::Keyword => Style::default().fg(Color::Blue).bold(),
                TokenKind::Function => Style::default().fg(Color::Magenta),
                TokenKind::DataType => Style::default().fg(Color::Cyan),
                TokenKind::StringLiteral => Style::default().fg(Color::Yellow),
                TokenKind::Number => Style::default().fg(Color::LightCyan),
                TokenKind::Comment => Style::default().fg(Color::DarkGray).italic(),
                TokenKind::Operator => Style::default().fg(Color::White),
                TokenKind::Identifier => Style::default().fg(Color::Green),
                TokenKind::Whitespace => Style::default(),
            };
            Span::styled(token.text, style)
        })
        .collect();

    Line::from(spans)
}

/// Highlight SQL for use in chat messages (with a prefix).
pub fn highlight_sql_with_prefix(prefix: &str, sql: &str) -> Line<'static> {
    let mut spans = vec![Span::styled(
        prefix.to_string(),
        Style::default().fg(Color::Green),
    )];

    let tokens = tokenize(sql);
    for token in tokens {
        let style = match token.kind {
            TokenKind::Keyword => Style::default().fg(Color::Blue).bold(),
            TokenKind::Function => Style::default().fg(Color::Magenta),
            TokenKind::DataType => Style::default().fg(Color::Cyan),
            TokenKind::StringLiteral => Style::default().fg(Color::Yellow),
            TokenKind::Number => Style::default().fg(Color::LightCyan),
            TokenKind::Comment => Style::default().fg(Color::DarkGray).italic(),
            TokenKind::Operator => Style::default().fg(Color::White),
            TokenKind::Identifier => Style::default().fg(Color::Green),
            TokenKind::Whitespace => Style::default(),
        };
        spans.push(Span::styled(token.text, style));
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_select() {
        let tokens = tokenize("SELECT * FROM users");
        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(kinds[0], TokenKind::Keyword); // SELECT
        assert_eq!(kinds[1], TokenKind::Whitespace);
        assert_eq!(kinds[2], TokenKind::Operator); // *
        assert_eq!(kinds[3], TokenKind::Whitespace);
        assert_eq!(kinds[4], TokenKind::Keyword); // FROM
        assert_eq!(kinds[5], TokenKind::Whitespace);
        assert_eq!(kinds[6], TokenKind::Identifier); // users
    }

    #[test]
    fn test_tokenize_string_literal() {
        let tokens = tokenize("WHERE name = 'hello'");
        let string_tokens: Vec<&Token> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::StringLiteral)
            .collect();
        assert_eq!(string_tokens.len(), 1);
        assert_eq!(string_tokens[0].text, "'hello'");
    }

    #[test]
    fn test_tokenize_number() {
        let tokens = tokenize("LIMIT 100");
        let num_tokens: Vec<&Token> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Number)
            .collect();
        assert_eq!(num_tokens.len(), 1);
        assert_eq!(num_tokens[0].text, "100");
    }

    #[test]
    fn test_tokenize_function() {
        let tokens = tokenize("SELECT COUNT(*) FROM users");
        let func_tokens: Vec<&Token> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Function)
            .collect();
        assert_eq!(func_tokens.len(), 1);
        assert_eq!(func_tokens[0].text, "COUNT");
    }

    #[test]
    fn test_tokenize_comment() {
        let tokens = tokenize("SELECT 1 -- this is a comment");
        let comment_tokens: Vec<&Token> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Comment)
            .collect();
        assert_eq!(comment_tokens.len(), 1);
        assert!(comment_tokens[0].text.starts_with("--"));
    }

    #[test]
    fn test_tokenize_escaped_string() {
        let tokens = tokenize("WHERE name = 'it''s'");
        let string_tokens: Vec<&Token> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::StringLiteral)
            .collect();
        assert_eq!(string_tokens.len(), 1);
        assert_eq!(string_tokens[0].text, "'it''s'");
    }

    #[test]
    fn test_tokenize_data_type() {
        let tokens = tokenize("CAST(id AS INTEGER)");
        let type_tokens: Vec<&Token> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::DataType)
            .collect();
        assert_eq!(type_tokens.len(), 1);
        assert_eq!(type_tokens[0].text, "INTEGER");
    }

    #[test]
    fn test_highlight_sql_returns_line() {
        let line = highlight_sql("SELECT * FROM users WHERE id = 1");
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_tokenize_complex_query() {
        let sql = "SELECT u.name, COUNT(o.id) FROM users u \
                   LEFT JOIN orders o ON u.id = o.user_id \
                   WHERE u.active = 1 GROUP BY u.name \
                   HAVING COUNT(o.id) > 5 ORDER BY u.name ASC LIMIT 10";
        let tokens = tokenize(sql);
        let keywords: Vec<&Token> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Keyword)
            .collect();
        // Should find: SELECT, FROM, LEFT, JOIN, ON, WHERE, GROUP, BY, HAVING, ORDER, BY, ASC, LIMIT
        assert!(keywords.len() >= 10);
    }
}
