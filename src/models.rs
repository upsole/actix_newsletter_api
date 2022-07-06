use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use dotenv::dotenv;

use actix_web::{web, HttpResponse};

use serde::{Deserialize, Serialize};

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

pub fn init_connection() -> PgConnection {
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set in .env");
    // PgConnection::establish(&db_url).expect("Error connection to Postgres")
    PgConnection::establish(&db_url).expect("Error connecting to Postgres")
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
}

fn list_accounts(conn: &DBPooledConnection) -> Result<Accounts, diesel::result::Error> {
    use super::schema::account::dsl::*;
    let _accounts_query = match account.load::<AccountDB>(conn)
    { 
            Ok(acts) => acts, 
            Err(_) => vec![],
    };
    Ok(Accounts {
        results: _accounts_query.into_iter().map(|a| a.to_account()).collect::<Vec<Account>>()
    })
}

pub async fn list(pool: web::Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect("Could not connect to DB");
    let accounts = web::block(move || list_accounts(&conn)).await.unwrap().unwrap();

    HttpResponse::Ok().content_type("application/json").json(accounts)
}

// TODO Error Handling for Already in use email
pub async fn post_account(input_account: web::Json<AccountRequest>) -> HttpResponse {
    let conn = init_connection();
    let new_account = input_account.to_account_db();
    diesel::insert_into(account::table)
        .values(&new_account)
        .execute(&conn)
        .expect("Insert failed");
    HttpResponse::Ok().json(new_account.to_account())
}
