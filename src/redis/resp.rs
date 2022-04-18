use std::{num::ParseIntError, str::FromStr};

pub enum RESP {
    SimpleStrings(SimpleStrings),
    Errors(Errors),
    Integers(Integers),
    BulkStrings(BulkStrings),
    Arrays(Arrays),
}

#[derive(Debug, Eq, PartialEq)]
pub struct SimpleStrings(Vec<u8>);

impl From<Vec<u8>> for SimpleStrings {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<String> for SimpleStrings {
    fn from(s: String) -> Self {
        Self::from(s.into_bytes())
    }
}

#[cfg(test)]
mod simple_strings_test {
    use super::*;

    #[test]
    fn from_test() {
        assert_eq!(
            SimpleStrings::from(vec![b'f', b'o', b'o']),
            SimpleStrings(vec![b'f', b'o', b'o'])
        );

        assert_eq!(
            SimpleStrings::from(String::from("foo")),
            SimpleStrings(vec![b'f', b'o', b'o'])
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
