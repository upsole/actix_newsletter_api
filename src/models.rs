use diesel::{AsChangeset, Identifiable, Insertable, Queryable};
use actix_web::web;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDateTime, Utc};
use uuid::Uuid;

use crate::schema::account;
use crate::domain::{ParseError, ParsedAccount, SanitizedEmail, SanitizedName};


#[derive(Debug, Deserialize, Serialize)]
pub struct Response<T> {
    pub results: Vec<T>,
}

impl<T> Response<T> {
    pub fn new() -> Self {
        Self { results: vec![] }
    }
}

pub type Accounts = Response<Account>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub id: String,
    pub subscribed_at: DateTime<Utc>,
    pub email: String,
    pub name: String,
    pub status: bool,
}

impl Account {
    pub fn new(id: String, email: String, name: String, status: bool) -> Self {
        Self {
            id,
            subscribed_at: Utc::now(),
            email,
            name,
            status,
        }
    }
}

// TODO BUG
#[derive(Deserialize, Queryable, Insertable, Identifiable, AsChangeset)]
#[table_name = "account"]
pub struct AccountDB {
    pub id: Uuid,
    pub subscribed_at: NaiveDateTime,
    pub email: String,
    pub name: String,
    pub status: bool,
    pub auth_token: Uuid,
}

impl AccountDB {
    pub fn to_account(&self) -> Account {
        Account::new(
            self.id.to_string(),
            self.email.clone(),
            self.name.clone(),
            self.status,
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountRequest {
    pub email: String,
    pub name: String,
}

impl AccountRequest {
    pub fn to_parsed_account(&self) -> Result<ParsedAccount, ParseError> {
        let san_name = SanitizedName::parse(self.name.clone())?;
        let san_email = SanitizedEmail::parse(self.email.clone())?;
        Ok(ParsedAccount {
            name: san_name,
            email: san_email,
        })
    }
}

impl TryFrom<web::Json<AccountRequest>> for ParsedAccount {
    type Error = ParseError;
    fn try_from(value: web::Json<AccountRequest>) -> Result<Self, Self::Error> {
        let name = SanitizedName::parse(value.name.clone())?;
        let email = SanitizedEmail::parse(value.email.clone())?;
        Ok(Self { name, email })
    }
}
