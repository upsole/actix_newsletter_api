use diesel::prelude::*;
use diesel::{Insertable, Queryable};

use actix_web::{web, HttpResponse};

use serde::{Deserialize, Serialize};

use crate::domain::{ParseError, ParsedAccount, SanitizedName, SanitizedEmail};
use crate::schema::account;
use crate::{DBPool, DBPooledConnection};

use chrono::{DateTime, NaiveDateTime, Utc};
use uuid::Uuid;

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

pub fn is_valid_name(s: &str) -> bool {
    let is_empty = s.trim().is_empty();
    let is_too_long = s.len() > 256;
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));

    !(is_empty || is_too_long || contains_forbidden_characters)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub id: String,
    pub subscribed_at: DateTime<Utc>,
    pub email: String,
    pub name: String,
    pub level: i32,
}

impl Account {
    pub fn new(email: String, name: String, level: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            subscribed_at: Utc::now(),
            email,
            name,
            level,
        }
    }
}

// TODO BUG
#[derive(Deserialize, Queryable, Insertable)]
#[table_name = "account"]
pub struct AccountDB {
    pub id: Uuid,
    pub subscribed_at: NaiveDateTime,
    pub email: String,
    pub name: String,
    pub level: i32,
}

impl AccountDB {
    pub fn to_account(&self) -> Account {
        Account::new(self.email.clone(), self.name.clone(), self.level)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountRequest {
    email: String,
    name: String,
    level: i32,
}

impl AccountRequest {
    pub fn to_account_db(&self) -> AccountDB {
        AccountDB {
            email: self.email.clone(),
            name: self.name.clone(),
            level: self.level,
            subscribed_at: Utc::now().naive_utc(),
            id: Uuid::new_v4(),
        }
    }
    pub fn to_parsed_account(&self) -> Result<ParsedAccount, ParseError> {
        let san_name = SanitizedName::parse(self.name.clone());
        let san_email = SanitizedEmail::parse(self.email.clone());
        match san_name {
            Ok(s) => {
                return Ok(ParsedAccount {
                    name: s,
                    email: match san_email {
                        Ok(s) => s,
                        Err(_) => return Err(ParseError),
                    },
                    level: self.level,
                })
            }
            Err(_) => return Err(ParseError),
        };
    }
}

fn list_accounts(conn: &DBPooledConnection) -> Result<Accounts, diesel::result::Error> {
    use super::schema::account::dsl::*;
    let _accounts_query = match account.load::<AccountDB>(conn) {
        Ok(acts) => acts,
        Err(_) => vec![],
    };
    Ok(Accounts {
        results: _accounts_query
            .into_iter()
            .map(|a| a.to_account())
            .collect::<Vec<Account>>(),
    })
}

fn create_account(
    input_account: ParsedAccount,
    conn: &DBPooledConnection,
) -> Result<Account, diesel::result::Error> {
    use crate::schema::account::dsl::*;
    let new_account = input_account.to_account_db();
    let _ = diesel::insert_into(account)
        .values(&new_account)
        .execute(conn)
        .expect("Insert failed");
    Ok(new_account.to_account())
}

#[tracing::instrument(name = "Querying and listing subscribers", skip(pool), fields())]
pub async fn list(pool: web::Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect("Could not connect to DB");
    let accounts = web::block(move || list_accounts(&conn))
        .await
        .unwrap()
        .unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .json(accounts)
}

// TODO Error Handling for Already in use email
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(pool),
    fields(
        subscriber_email = %input_account.email,
        subscriber_name = %input_account.name
    )
)]
pub async fn create(
    input_account: web::Json<AccountRequest>,
    pool: web::Data<DBPool>,
) -> HttpResponse {
    let parsed_account = input_account.to_parsed_account();
    let parsed_account = match parsed_account {
        Ok(act) => act,
        Err(_) => {
            tracing::error!("Invalid payload in request");
            return HttpResponse::BadRequest().finish();
        }
    };

    let conn = pool.get().expect("Could not connect to DB");
    let new_acct = web::block(move || create_account(parsed_account, &conn))
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute insertion {:?}", e);
        });

    match new_acct {
        Ok(new_acct) => HttpResponse::Created()
            .content_type("application/json")
            // TODO fix
            .json(new_acct.unwrap()),
        Err(e) => {
            tracing::error!("Failed to execute insertion {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
