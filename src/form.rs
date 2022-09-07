use anyhow::Error;
use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq)]
pub enum Field {
    Op,
    Email,
    Subject,
    Body,
    Files,
}

pub struct Validator {
    errors: HashMap<Field, Error>,
}

impl Validator {
    pub async fn valid(&self) -> bool {
        self.errors.values().all(|r| r.is::<Error>())
    }

    pub async fn check(&mut self, ok: bool, k: Field, v: Error) {
        if !ok {
            self.errors.insert(k, v);
        }
    }
}
