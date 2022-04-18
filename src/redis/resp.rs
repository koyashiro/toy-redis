use std::error::Error;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

pub enum RESP {
    SimpleStrings(SimpleStrings),
    Errors(Errors),
    Integers(Integers),
    BulkStrings(BulkStrings),
    Arrays(Arrays),
}

#[derive(Debug, Eq, PartialEq)]
pub struct SimpleStrings(Vec<u8>);

impl TryFrom<Vec<u8>> for SimpleStrings {
    type Error = TryFromSimpleStringsError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.contains(&b'\r') || value.contains(&b'\n') {
            return Err(TryFromSimpleStringsError());
        }
        Ok(Self(value))
    }
}

impl TryFrom<String> for SimpleStrings {
    type Error = TryFromSimpleStringsError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.into_bytes())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TryFromSimpleStringsError();

impl TryFromSimpleStringsError {
    #[doc(hidden)]
    pub fn __description(&self) -> &str {
        "newline (the 0xA and 0xD byte) are not allowed"
    }
}

impl Display for TryFromSimpleStringsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.__description())
    }
}

impl Error for TryFromSimpleStringsError {}

#[cfg(test)]
mod simple_strings_test {
    use super::*;

    #[test]
    fn from_test() {
        assert_eq!(
            SimpleStrings::try_from(vec![b'f', b'o', b'o']),
            Ok(SimpleStrings(vec![b'f', b'o', b'o']))
        );

        assert_eq!(
            SimpleStrings::try_from(vec![b'f', b'o', b'o', b'\r', b'b', b'a', b'r']),
            Err(TryFromSimpleStringsError())
        );

        assert_eq!(
            SimpleStrings::try_from(vec![b'f', b'o', b'o', b'\n', b'b', b'a', b'r']),
            Err(TryFromSimpleStringsError())
        );

        assert_eq!(
            SimpleStrings::try_from(String::from("foo")),
            Ok(SimpleStrings(vec![b'f', b'o', b'o']))
        );

        assert_eq!(
            SimpleStrings::try_from(String::from("foo\rbar")),
            Err(TryFromSimpleStringsError())
        );

        assert_eq!(
            SimpleStrings::try_from(String::from("foo\nbar")),
            Err(TryFromSimpleStringsError())
        );
    }
}

#[derive(Debug, Eq, PartialEq)]
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

#[derive(Debug, Eq, PartialEq)]
pub struct Integers(i64);

impl From<i64> for Integers {
    fn from(i: i64) -> Self {
        Self(i)
    }
}

impl FromStr for Integers {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let i: i64 = s.parse()?;
        Ok(Self(i))
    }
}

#[cfg(test)]
mod integers_test {
    use super::*;

    #[test]
    fn from_test() {
        assert_eq!(Integers::from(123_i64), Integers(123_i64));

        assert_eq!("123".parse::<Integers>(), Ok(Integers(123_i64)));

        assert_eq!("invalid".parse::<Integers>(), "invalid".parse());

        assert_eq!(Integers::from(-123_i64), Integers(-123_i64));

        assert_eq!("-123".parse::<Integers>(), Ok(Integers(-123_i64)));
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct BulkStrings(Vec<u8>);

impl From<Vec<u8>> for BulkStrings {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<String> for BulkStrings {
    fn from(s: String) -> Self {
        Self::from(s.into_bytes())
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

#[derive(Debug, Eq, PartialEq)]
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
