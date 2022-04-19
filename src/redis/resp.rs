use std::fmt;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RESP {
    SimpleStrings(String),
    Errors(String),
    Integers(i64),
    BulkStrings(Option<Vec<u8>>),
    Arrays(Option<Vec<RESP>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleStrings(Vec<u8>);

impl SimpleStrings {
    pub fn new(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl TryFrom<Vec<u8>> for SimpleStrings {
    type Error = TryFromSimpleStringsError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(TryFromSimpleStringsError {
                kind: SimpleStringsErrorKind::Empty,
            });
        }

        if !value.starts_with(b"+") {
            return Err(TryFromSimpleStringsError {
                kind: SimpleStringsErrorKind::InvalidPrefix,
            });
        }

        if value[1..value.len() - 2].contains(&b'\r') || value[1..value.len() - 2].contains(&b'\n')
        {
            return Err(TryFromSimpleStringsError {
                kind: SimpleStringsErrorKind::ContainsNewLine,
            });
        }

        if !value.ends_with(b"\r\n") {
            return Err(TryFromSimpleStringsError {
                kind: SimpleStringsErrorKind::InvalidSuffix,
            });
        }

        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TryFromSimpleStringsError {
    kind: SimpleStringsErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SimpleStringsErrorKind {
    Empty,
    InvalidPrefix,
    ContainsNewLine,
    InvalidSuffix,
}

impl TryFromSimpleStringsError {
    #[doc(hidden)]
    pub fn __description(&self) -> &str {
        match self.kind {
            SimpleStringsErrorKind::Empty => "cannot parse float from empty string",
            SimpleStringsErrorKind::InvalidPrefix => "must start with `+` (the 0x2B byte)",
            SimpleStringsErrorKind::InvalidSuffix => {
                "must end with `\\r\\n` (the 0xA and 0xD byte)"
            }
            SimpleStringsErrorKind::ContainsNewLine => {
                "newline (the 0xA and 0xD byte) are not allowed"
            }
        }
    }
}

impl Display for TryFromSimpleStringsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.__description().fmt(f)
    }
}

#[cfg(test)]
mod simple_strings_test {
    use super::*;

    #[test]
    fn try_from_test() {
        assert_eq!(
            SimpleStrings::try_from(vec![b'+', b'a', b'b', b'c', b'\r', b'\n']),
            Ok(SimpleStrings(vec![b'+', b'a', b'b', b'c', b'\r', b'\n']))
        );

        assert_eq!(
            SimpleStrings::try_from(vec![]),
            Err(TryFromSimpleStringsError {
                kind: SimpleStringsErrorKind::Empty
            })
        );

        assert_eq!(
            SimpleStrings::try_from(vec![b'a', b'b', b'c', b'\r', b'\n']),
            Err(TryFromSimpleStringsError {
                kind: SimpleStringsErrorKind::InvalidPrefix
            })
        );

        assert_eq!(
            SimpleStrings::try_from(vec![b'+', b'a', b'\r', b'c', b'\r', b'\n']),
            Err(TryFromSimpleStringsError {
                kind: SimpleStringsErrorKind::ContainsNewLine
            })
        );

        assert_eq!(
            SimpleStrings::try_from(vec![b'+', b'a', b'\n', b'c', b'\r', b'\n']),
            Err(TryFromSimpleStringsError {
                kind: SimpleStringsErrorKind::ContainsNewLine
            })
        );

        assert_eq!(
            SimpleStrings::try_from(vec![b'+', b'a', b'b', b'c']),
            Err(TryFromSimpleStringsError {
                kind: SimpleStringsErrorKind::InvalidSuffix
            })
        );

        assert_eq!(
            SimpleStrings::try_from(vec![b'+', b'a', b'b', b'c', b'\r']),
            Err(TryFromSimpleStringsError {
                kind: SimpleStringsErrorKind::InvalidSuffix
            })
        );

        assert_eq!(
            SimpleStrings::try_from(vec![b'+', b'a', b'b', b'c', b'\n']),
            Err(TryFromSimpleStringsError {
                kind: SimpleStringsErrorKind::InvalidSuffix
            })
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Errors(Vec<u8>);

impl From<Vec<u8>> for Errors {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<String> for Errors {
    fn from(s: String) -> Self {
        Self::from(s.into_bytes())
    }
}

#[cfg(test)]
mod errors_test {
    use super::*;

    #[test]
    fn from_test() {
        assert_eq!(
            Errors::from(vec![b'e', b'r', b'r']),
            Errors(vec![b'e', b'r', b'r'])
        );

        assert_eq!(
            Errors::from(String::from("err")),
            Errors(vec![b'e', b'r', b'r'])
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Integers(i64);

impl From<i64> for Integers {
    fn from(i: i64) -> Self {
        Self(i)
    }
}

impl FromStr for Integers {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let i = i64::from_str(s)?;
        Ok(Self(i))
    }
}

#[cfg(test)]
mod integers_test {
    use super::*;

    #[test]
    fn from_test() {
        assert_eq!(Integers::from(123_i64), Integers(123_i64));
        assert_eq!(Integers::from(-123_i64), Integers(-123_i64));
    }

    #[test]
    fn from_str_test() {
        assert_eq!(Integers::from_str("123"), Ok(Integers(123_i64)));
        assert_eq!(Integers::from_str("-123"), Ok(Integers(-123_i64)));
        assert_eq!(Integers::from_str("invalid"), "invalid".parse());
    }
}

const NIL_BULK_STRINGS: [u8; 5] = [b'$', b'-', b'1', b'\r', b'\n'];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkStrings(Vec<u8>);

impl BulkStrings {
    pub fn nil() -> Self {
        Self(NIL_BULK_STRINGS.to_vec())
    }

    pub fn is_nil(&self) -> bool {
        self.0 == NIL_BULK_STRINGS
    }
}

impl From<Vec<u8>> for BulkStrings {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<String> for BulkStrings {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

#[cfg(test)]
mod bulk_strings_test {
    use super::*;

    #[test]
    fn from_test() {
        assert_eq!(
            BulkStrings::from(vec![b'f', b'o', b'o']),
            BulkStrings(vec![b'f', b'o', b'o'])
        );

        assert_eq!(
            BulkStrings::from(String::from("foo")),
            BulkStrings(vec![b'f', b'o', b'o'])
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arrays(Vec<BulkStrings>);

impl From<Vec<BulkStrings>> for Arrays {
    fn from(v: Vec<BulkStrings>) -> Self {
        Self(v)
    }
}

#[cfg(test)]
mod arrays_test {
    use super::*;

    #[test]
    fn from_test() {
        assert_eq!(
            Arrays::from(vec![BulkStrings::from(String::from("foo")),]),
            Arrays(vec![BulkStrings(vec![b'f', b'o', b'o']),])
        );

        assert_eq!(
            Arrays::from(vec![
                BulkStrings::from(String::from("foo")),
                BulkStrings::from(String::from("bar")),
            ]),
            Arrays(vec![
                BulkStrings(vec![b'f', b'o', b'o']),
                BulkStrings(vec![b'b', b'a', b'r']),
            ])
        );
    }
}
