use std::fmt::format;

use crate::query::date::*;
use crate::query::lexer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QueryModifiersTracking {
    pub case_sensitive: bool,
    pub diacritics_sensitive: bool,
    pub file_only: bool,
    pub folder_only: bool,
    pub match_path: bool,
    pub regex: bool,
    pub whole_filename: bool,
    pub whole_word: bool,
    pub wildcards: bool,
}

impl Default for QueryModifiersTracking {
    fn default() -> Self {
        QueryModifiersTracking {
            case_sensitive: false,
            diacritics_sensitive: false,
            file_only: false,
            folder_only: false,
            match_path: false,
            regex: false,
            whole_filename: false,
            whole_word: false,
            wildcards: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextQuery {
    pub text: String,
    pub case_sensitive: bool,
    pub diacritics_sensitive: bool,
    pub file_only: bool,
    pub folder_only: bool,
    pub match_path: bool,
    pub whole_filename: bool,
    pub whole_word: bool,
}

#[derive(Debug, Clone)]
pub struct RegexQuery {
    pub pattern: regex::Regex,
    pub case_sensitive: bool,
    pub diacritics_sensitive: bool,
    pub match_path: bool,
}

#[derive(Debug, Clone)]
pub enum QueryLiteral {
    Text(TextQuery),
    Regex(RegexQuery),
}

#[derive(Debug, Clone)]
pub enum QueryExpr {
    Literal(QueryLiteral),
    Function(QueryFunction),
    And(Box<QueryExpr>, Box<QueryExpr>),
    Or(Box<QueryExpr>, Box<QueryExpr>),
    Not(Box<QueryExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryFunction {
    Size(QueryCmp, u64),
    DateModified(QueryCmp, QueryDate),
    DateCreated(QueryCmp, QueryDate),
    Parent(String),
    Ext(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryCmp {
    Eq,
    Gt,
    Ge,
    Lt,
    Le,
    Range, // start..end
}
impl From<&str> for QueryCmp {
    fn from(s: &str) -> Self {
        match s {
            "=" => QueryCmp::Eq,
            ">" => QueryCmp::Gt,
            ">=" => QueryCmp::Ge,
            "<" => QueryCmp::Lt,
            "<=" => QueryCmp::Le,
            ".." => QueryCmp::Range,
            _ => QueryCmp::Eq, // Default to Eq if unknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Sunday = 0,
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}
#[derive(Debug, Clone, PartialEq)]
pub enum QueryDate {
    Range(i64, i64),  // start, end as timestamps
    Weekday(Weekday), // 0=Sun - 6=Sat
    Month(Month),     // 1=Jan - 12=Dec
    Unknown,
}

fn exprs_to_and(exprs: Vec<QueryExpr>) -> QueryExpr {
    if exprs.is_empty() {
        return QueryExpr::Literal(QueryLiteral::Text(TextQuery {
            text: "".to_string(),
            case_sensitive: false,
            diacritics_sensitive: false,
            file_only: false,
            folder_only: false,
            match_path: false,
            whole_filename: false,
            whole_word: false,
        }));
    }
    let mut iter = exprs.into_iter();
    let first = iter.next().unwrap();
    iter.fold(first, |acc, expr| {
        QueryExpr::And(Box::new(acc), Box::new(expr))
    })
}

fn get_comparison(lexer: &mut lexer::QueryLexer) -> Option<QueryCmp> {
    if let Some(token) = lexer.peek_token() {
        match token {
            lexer::QueryToken::Equal => {
                lexer.next_token();
                Some(QueryCmp::Eq)
            }
            lexer::QueryToken::GreaterThan => {
                lexer.next_token();
                Some(QueryCmp::Gt)
            }
            lexer::QueryToken::GreaterThanOrEqual => {
                lexer.next_token();
                Some(QueryCmp::Ge)
            }
            lexer::QueryToken::LessThan => {
                lexer.next_token();
                Some(QueryCmp::Lt)
            }
            lexer::QueryToken::LessThanOrEqual => {
                lexer.next_token();
                Some(QueryCmp::Le)
            }
            _ => Some(QueryCmp::Eq),
        }
    } else {
        None
    }
}

fn create_query_literal(text: String, modifiers: QueryModifiersTracking) -> QueryLiteral {
    if modifiers.regex {
        // Create RegexQuery
        let mut regex_builder = regex::RegexBuilder::new(&text);
        if !modifiers.case_sensitive {
            regex_builder.case_insensitive(true);
        }
        let pattern = regex_builder
            .build()
            .unwrap_or_else(|_| regex::Regex::new(".*").unwrap());
        QueryLiteral::Regex(RegexQuery {
            pattern,
            case_sensitive: modifiers.case_sensitive,
            diacritics_sensitive: modifiers.diacritics_sensitive,
            match_path: modifiers.match_path,
        })
    } else {
        // Create TextQuery
        QueryLiteral::Text(TextQuery {
            text,
            case_sensitive: modifiers.case_sensitive,
            diacritics_sensitive: modifiers.diacritics_sensitive,
            file_only: modifiers.file_only,
            folder_only: modifiers.folder_only,
            match_path: modifiers.match_path,
            whole_filename: modifiers.whole_filename,
            whole_word: modifiers.whole_word,
        })
    }
}

// Parses a function like size:>1000 or datecreated:<2023-01-01
fn parse_function(lexer: &mut lexer::QueryLexer, name: &str) -> Option<QueryFunction> {
    let name = name.to_lowercase();
    let name = name.as_str();
    match name {
        "size" => {
            let cmp = get_comparison(lexer)?;
            if let Some(token) = lexer.next_token() {
                match token {
                    lexer::QueryToken::Ident(num_str) | lexer::QueryToken::StrLit(num_str) => {
                        if let Ok(size) = num_str.parse::<u64>() {
                            return Some(QueryFunction::Size(cmp, size));
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        "datemodified" | "dm" | "datecreated" | "dc" => {
            let cmp = get_comparison(lexer)?;
            if let Some(token) = lexer.next_token() {
                match token {
                    lexer::QueryToken::Ident(date_str) | lexer::QueryToken::StrLit(date_str) => {
                        // use QueryDate::from
                        let date = QueryDate::from(date_str.as_str());
                        return Some(if name.starts_with("datecreated") || name == "dc" {
                            QueryFunction::DateCreated(cmp, date)
                        } else {
                            QueryFunction::DateModified(cmp, date)
                        });
                    }
                    _ => {}
                }
            }
            None
        }
        "parent" | "infolder" | "nosubfolders" => {
            if let Some(token) = lexer.next_token() {
                match token {
                    lexer::QueryToken::Ident(folder) | lexer::QueryToken::StrLit(folder) => {
                        return Some(QueryFunction::Parent(folder));
                    }
                    _ => {}
                }
            }
            None
        }
        "ext" => {
            let mut exts = Vec::new();
            while let Some(token) = lexer.next_token() {
                match token {
                    lexer::QueryToken::Ident(ext) | lexer::QueryToken::StrLit(ext) => {
                        exts.push(ext);
                    }
                    _ => {}
                }
            }
            if !exts.is_empty() {
                return Some(QueryFunction::Ext(exts));
            }
            None
        }
        _ => None,
    }
}

// Parses a modifier like case:query or file:query
fn parse_modifier(
    ident: &String,
    modifiers: QueryModifiersTracking,
) -> Option<QueryModifiersTracking> {
    let mut modifiers = modifiers;

    let lower_ident = ident.to_lowercase();
    match lower_ident.as_str() {
        "case" => modifiers.case_sensitive = true,
        "nocase" => modifiers.case_sensitive = false,
        "diacritics" => modifiers.diacritics_sensitive = true,
        "nodiacritics" => modifiers.diacritics_sensitive = false,
        "file" | "files" => {
            modifiers.file_only = true;
            modifiers.folder_only = false;
        }
        "nofileonly" => modifiers.file_only = false,
        "folder" | "folders" => {
            modifiers.folder_only = true;
            modifiers.file_only = false;
        }
        "nofolderonly" => modifiers.folder_only = false,
        "path" => modifiers.match_path = true,
        "nopath" => modifiers.match_path = false,
        "regex" => modifiers.regex = true,
        "noregex" => modifiers.regex = false,
        "wholefilename" | "wfn" | "exact" => modifiers.whole_filename = true,
        "nowfn" | "nowholefilename" => modifiers.whole_filename = false,
        "wholeword" | "ww" => modifiers.whole_word = true,
        "nowholeword" | "noww" => modifiers.whole_word = false,
        "wildcards" => modifiers.wildcards = true,
        "nowildcards" => modifiers.wildcards = false,
        _ => {
            return None; // Not a modifier
        }
    }

    Some(modifiers)
}

// Parses a single condition, which could be a function, a text query, or a negation
// e.g. size:>1000, "example.txt", file:case:"ExAmplE.txt", !ext:tmp
// extreme cases: !case:!file:"!"tmp  // double negation with query !tmp
fn parse_condition(lexer: &mut lexer::QueryLexer, modifiers: QueryModifiersTracking) -> QueryExpr {
    if let Some(token) = lexer.next_token() {
        match token {
            lexer::QueryToken::Ident(ref ident) => {
                // Check if next token is Colon for function
                if let Some(lexer::QueryToken::Colon) = lexer.peek_token() {
                    // Consume Colon
                    lexer.next_token();
                    // Try parse function
                    if let Some(func) = parse_function(lexer, &ident) {
                        return QueryExpr::Function(func);
                    } else if let Some(new_modifiers) =
                        parse_modifier(&ident, modifiers)
                    {
                        // If it's a modifier, update modifiers and continue
                        return parse_condition(lexer, new_modifiers);
                    }
                }
                // Otherwise, treat as text query
            }
            lexer::QueryToken::Not => {
                let sub_expr = parse_condition(lexer, modifiers);
                return QueryExpr::Not(Box::new(sub_expr));
            }
            lexer::QueryToken::Whitespace => {
                unreachable!("Whitespace should be handled in parse_expression");
            }
            lexer::QueryToken::LessThan => {
                // start of block
                return parse_expression(lexer, modifiers);
            }
            _ => {
                // Otherwise, treat as text query
            }
        };

        let mut search_text = token.to_string();
        while let Some(next_token) = lexer.peek_token() {
            match next_token {
                lexer::QueryToken::Whitespace | lexer::QueryToken::Or => break,
                _ => {
                    // Consume token and append to search_text
                    if let Some(t) = lexer.next_token() {
                        search_text.push_str(&t.to_string());
                    }
                }
            }
        }
        let literal = create_query_literal(search_text, modifiers);
        return QueryExpr::Literal(literal);
    }
    // Default to empty text query if nothing matched
    QueryExpr::Literal(QueryLiteral::Text(TextQuery {
        text: "".to_string(),
        case_sensitive: false,
        diacritics_sensitive: false,
        file_only: false,
        folder_only: false,
        match_path: false,
        whole_filename: false,
        whole_word: false,
    }))
}

fn parse_expression(lexer: &mut lexer::QueryLexer, modifiers: QueryModifiersTracking) -> QueryExpr {
    let mut exprs = Vec::new();
    while let Some(token) = lexer.peek_token() {
        match token {
            lexer::QueryToken::Whitespace => {
                // Just skip whitespace
                lexer.next_token();
            }
            lexer::QueryToken::Or => {
                // Parse next condition and combine with Or
                lexer.next_token(); // consume Or
                let right_expr = parse_expression(lexer, modifiers);
                let left_expr = exprs_to_and(exprs);
                return QueryExpr::Or(Box::new(left_expr), Box::new(right_expr));
            }
            _ => {
                exprs.push(parse_condition(lexer, modifiers));
            }
        }
    }
    exprs_to_and(exprs)
}

pub fn parse_query(input: &str) -> QueryExpr {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        // Return a default empty query or handle as needed
        return QueryExpr::Literal(QueryLiteral::Text(TextQuery {
            text: "".to_string(),
            case_sensitive: false,
            diacritics_sensitive: false,
            file_only: false,
            folder_only: false,
            match_path: false,
            whole_filename: false,
            whole_word: false,
        }));
    }

    let mut lexer = lexer::QueryLexer::new(input);
    let modifiers = QueryModifiersTracking::default();
    parse_expression(&mut lexer, modifiers)
}
