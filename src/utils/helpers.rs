use chrono::Datelike;

fn current_year() -> u32 {
    chrono::Utc::now().year() as u32
}
