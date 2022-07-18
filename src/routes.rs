use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::ParsedAccount;
use crate::email::EmailClient;
use crate::{DBPool, DBPooledConnection};

use crate::models::{Account, AccountDB, AccountRequest, Accounts};

// GET /subscriptions
// TODO needs auth
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

// POST subscriptions
// Creates acct
fn create_account(
    input_account: ParsedAccount,
    conn: &DBPooledConnection,
) -> Result<AccountDB, diesel::result::Error> {
    use crate::schema::account::dsl::*;
    let new_account = input_account.to_account_db();
    let _ = diesel::insert_into(account)
        .values(&new_account)
        .execute(conn)
        .expect("Insert failed");
    Ok(new_account)
}

// TODO Error Handling for Already in use email
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(pool, email_client),
    fields(
        subscriber_email = %input_account.email,
        subscriber_name = %input_account.name
    )
)]
pub async fn create(
    input_account: web::Json<AccountRequest>,
    pool: web::Data<DBPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let parsed_account: ParsedAccount = match input_account.try_into() {
        Ok(values) => values,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let clone_acct = parsed_account.clone();

    let conn = pool.get().expect("Could not connect to DB");
    let new_acct = web::block(move || create_account(parsed_account, &conn))
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute insertion {:?}", e);
        });

    match new_acct {
        Ok(new_acct) => {
            match email_client.send_confirmation(clone_acct, new_acct.as_ref().unwrap().auth_token)
            {
                Ok(_) => HttpResponse::Created()
                    .content_type("application/json")
                    .json(new_acct.unwrap().to_account()),
                Err(_) => HttpResponse::InternalServerError().finish()
            }
        }
        Err(e) => {
            tracing::error!("Failed to execute insertion {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// POST /confirm/{req_token}
// Activates account
#[derive(Serialize)]
struct ActivateResponse {
    message: String,
}
struct ActivateError;
fn activate_account(
    req_token: &str,
    conn: &DBPooledConnection,
) -> Result<ActivateResponse, ActivateError> {
    use crate::schema::account::dsl::*;
    let updated_account =
        diesel::update(account.filter(auth_token.eq(Uuid::parse_str(req_token).unwrap())))
            .set(status.eq(true))
            .get_result::<AccountDB>(conn);

    match updated_account {
        Ok(act) => Ok(ActivateResponse {
            message: format!("Confirmed email account! {}", act.email),
        }),
        Err(_) => Err(ActivateError),
    }
}

pub async fn confirm(path: web::Path<String>, pool: web::Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect("Could not connect to DB");
    let req_token = path.into_inner();
    let updated_account = web::block(move || activate_account(&req_token, &conn))
        .await
        .unwrap();
    match updated_account {
        Ok(act) => HttpResponse::Ok().json(act),
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}

// TODO /send newsletter
