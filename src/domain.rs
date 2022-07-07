use crate::models::AccountDB;
use uuid::Uuid;
use chrono::Utc;

use std::fmt;

pub struct SanitizedName(String);
pub struct InvalidNameError;

impl SanitizedName {
    pub fn parse(s: String) -> Result<SanitizedName, InvalidNameError> {
        let is_empty = s.trim().is_empty();
        let is_too_long = s.len() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));

        if is_empty || is_too_long || contains_forbidden_characters {
            return Err(InvalidNameError)
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

pub struct ParsedAccount {
    pub email: String,
    pub name: SanitizedName,
    pub level: i32,
}

impl ParsedAccount {
    pub fn to_account_db(&self) -> AccountDB {
        AccountDB {
            email: self.email.clone(),
            // TODO figure out SanName -> String
            name: self.name.to_string(),
            level: self.level,
            subscribed_at: Utc::now().naive_utc(),
            id: Uuid::new_v4(),
        }
    }
}
