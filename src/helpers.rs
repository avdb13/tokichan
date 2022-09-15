use std::str::from_utf8;

use anyhow::Result;
use regex::Regex;
use tracing::{event, Level};

pub fn validate_upload_limit(s: String) -> Result<usize> {
    let re = Regex::new(r"^(?P<num>\d+)\s*(?:(?P<prefix>[KMG])(?P<base>i)?)?B?$")?;
    match re.is_match(&s) {
        false => Err(regex::Error::Syntax(s).into()),
        true => {
            event!(Level::INFO, "Regex is correct!");

            let n = ['K', 'M', 'G']
                .iter()
                .position(|&c| c == s.as_bytes()[s.len() - 1] as char)
                .unwrap();
            Ok(from_utf8(&s.as_bytes()[0..s.len() - 2])
                .unwrap()
                .parse::<usize>()
                .unwrap()
                * 1024_usize.pow(n.try_into().unwrap()))
        }
    }
}
