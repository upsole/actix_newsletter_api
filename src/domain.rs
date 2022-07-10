use crate::models::AccountDB;
use uuid::Uuid;
use chrono::Utc;
use validator::validate_email;

use std::fmt;

pub struct SanitizedName(String);
#[derive(Debug)]
pub struct ParseError;

impl SanitizedName {
    pub fn parse(s: String) -> Result<SanitizedName, ParseError> {
        let is_empty = s.trim().is_empty();
        let is_too_long = s.len() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));

        if is_empty || is_too_long || contains_forbidden_characters {
            return Err(ParseError)
        } else {
            Ok(Self(s))
        }
    }
}

impl fmt::Display for SanitizedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone)]
pub struct SanitizedEmail(String);

impl AsRef<str> for SanitizedEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SanitizedEmail {
    pub fn parse(s: String) -> Result<SanitizedEmail, ParseError> {
        let is_empty = s.trim().is_empty();
        let is_too_long = s.len() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));
        // TODO Regex email validation

        if is_empty || is_too_long || contains_forbidden_characters ||!validate_email(&s) {
            return Err(ParseError)
        } else {
            Ok(Self(s))
        }

    }
}

impl fmt::Display for SanitizedEmail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}



pub struct ParsedAccount {
    pub email: SanitizedEmail,
    pub name: SanitizedName,
    pub level: i32,
}

impl ParsedAccount {
    pub fn to_account_db(&self) -> AccountDB {
        AccountDB {
            email: self.email.to_string(),
            name: self.name.to_string(),
            level: self.level,
            subscribed_at: Utc::now().naive_utc(),
            id: Uuid::new_v4(),
        }
    }
}
