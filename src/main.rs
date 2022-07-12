use actix_web::{web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use email_api::{
    routes,
    telemetry::{get_subscriber, init_subscriber},
};

use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;

use tracing_actix_web::TracingLogger;

async fn ok() -> HttpResponse {
    HttpResponse::Ok().body("Server is running.\n")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set in .env");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
    // let email_client = EmailClient::new(email_api, sender_email, auth_token, timeout);

    // Inits subscriber with default level = info. We can change log level by exporting RUST_LOG
    // env variable
    let subscriber = get_subscriber("email-api".into(), "info".into());
    init_subscriber(subscriber);

    // TODO init connection poo and refactor to pass it to routes
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(pool.clone()))
            // .app_data(web::Data::new(email_client.clone()))
            .route("/ok", web::get().to(ok))
            .route("/subscriptions", web::get().to(routes::list))
            .route("/subscriptions", web::post().to(routes::create))
            .route("/confirm/{req_token}", web::post().to(routes::confirm))
            // .route("/test", web::post().to(models::send_email))
    })
    .bind("127.0.0.1:4000")?
    .run()
    .await
}
