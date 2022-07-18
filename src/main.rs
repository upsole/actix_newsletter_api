use actix_web::{web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use email_api::{
    domain::SanitizedEmail,
    routes,
    telemetry::{get_subscriber, init_subscriber},
};

use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;

use tracing_actix_web::TracingLogger;

use email_api::email::EmailClient;

async fn ok() -> HttpResponse {
    HttpResponse::Ok().body("Server is running.\n")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Environment variables
    // Database
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set in .env");

    // SMTP Server
    let base_url = std::env::var("BASE_URL").expect("BASE_URL not set in .env");
    let sender_email = std::env::var("SENDER_EMAIL").expect("BASE_URL not set in .env");
    let smtp_url = std::env::var("SMTP_URL").expect("SMTP_URL not set in .env");
    let smtp_user = std::env::var("SMTP_USERNAME").expect("SMTP_USERNAME not set in .env");
    let smtp_password = std::env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD not set in .env");

    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    let sender_email =
        SanitizedEmail::parse(sender_email).expect("Bad sender email format");
    let email_client = EmailClient::new(
        smtp_url,
        sender_email,
        smtp_user,
        smtp_password,
        base_url,
    ).expect("Failed to initialize client");

    // Inits subscriber with default level = info. We can change log level by exporting RUST_LOG
    // env variable
    let subscriber = get_subscriber("email-api".into(), "info".into());
    init_subscriber(subscriber);

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(email_client.clone()))
            .route("/ok", web::get().to(ok))
            .route("/subscriptions", web::get().to(routes::list))
            .route("/subscriptions", web::post().to(routes::create))
            .route("/confirm/{req_token}", web::get().to(routes::confirm))
    })
    .bind("127.0.0.1:4000")?
    .run()
    .await
}
