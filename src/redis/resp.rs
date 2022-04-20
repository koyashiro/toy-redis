#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RESP {
    SimpleStrings(String),
    Errors(String),
    Integers(i64),
    BulkStrings(Option<Vec<u8>>),
    Arrays(Option<Vec<RESP>>),
}
