use anyhow::Result;
use regex::Regex;
use serde_derive::Deserialize;
use std::str::from_utf8;
use tracing::{event, Level};

#[derive(Deserialize)]
pub struct Config {
    pub psql: Psql,
    pub security: Security,
}

#[derive(Deserialize)]
pub struct Psql {
    pub username: String,
    pub password: String,
    pub address: String,
}

#[derive(Deserialize)]
pub struct Security {
    pub upload_limit: String,
    pub allowed_mimes: Vec<String>,
}

impl Security {
    pub fn validate_upload_limit(&self) -> Result<usize> {
        let s = &self.upload_limit;
        let re = Regex::new(r"^(?P<num>\d+)\s*(?:(?P<prefix>[KMG])(?P<base>i)?)?B?$")?;

        match re.is_match(s) {
            false => Err(regex::Error::Syntax(s.to_string()).into()),
            true => {
                event!(Level::INFO, "Regex is correct!");

                let n = ['K', 'M', 'G']
                    .iter()
                    .position(|&c| c == s.as_bytes()[s.len() - 1] as char)
                    .unwrap();
                Ok(Some(&s[0..s.len() - 2]).unwrap().parse::<usize>().unwrap()
                    * 1024_usize.pow(n.try_into().unwrap()))
            }
        }
    }
}
