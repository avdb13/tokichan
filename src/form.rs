use std::collections::HashMap;

enum Field {
    Op,
    Email,
    Subject,
    Body,
    Files,
}

pub struct Validator<T, E> {
    errors: HashMap<Field, Vec<Result<T, E>>>,
}

impl Validator<T, E> {
    pub async fn valid(&self) -> bool {
        self.errors.iter().all(|r| r.is_ok())
    }

    pub async fn 
}
