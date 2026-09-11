use chrono::{Local, NaiveDateTime, Utc};

pub fn get_current_time_stamp() ->String{
    Local::now().to_string()
}

pub fn get_time_naive()->NaiveDateTime{
    Local::now().naive_local()
}

pub fn is_all_lowercase(s: &str) -> bool {
    s.chars().filter(|c| c.is_alphabetic()).all(|c| c.is_lowercase())
}

pub fn has_no_spaces(s: &str) -> bool {
    !s.contains(' ') 
}