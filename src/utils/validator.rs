

extern crate regex;
use regex::Regex;


// take in an email address and checks if its a valid email
fn validate_email(email:String) -> bool{
    let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
    return email_regex.is_match(email.as_str())
}