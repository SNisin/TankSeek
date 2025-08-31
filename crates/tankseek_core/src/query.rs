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
    Path(String),
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
