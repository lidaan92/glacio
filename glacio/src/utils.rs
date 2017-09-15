use {Error, Result};
use regex::Captures;
use std::str::FromStr;

pub fn parse_capture<T>(captures: &Captures, name: &str) -> Result<T>
    where T: FromStr,
          Error: From<<T as FromStr>::Err>
{
    captures.name(name)
        .unwrap()
        .as_str()
        .parse()
        .map_err(Error::from)
}
