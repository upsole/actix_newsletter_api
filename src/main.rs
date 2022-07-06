use actix_web::{web, App, HttpResponse, HttpServer};
use email_api::models;
use dotenv::dotenv;

use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;
// pub type DBPool = Pool<ConnectionManager<PgConnection>>;
// pub type DBPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;


async fn ok() -> HttpResponse {
    HttpResponse::Ok().body("Server is running.\n")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set in .env");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder().build(manager).expect("Failed to create pool");
    println!("Server starting...");

    // TODO init connection poo and refactor to pass it to routes
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/ok", web::get().to(ok))
            .route("/subscriptions", web::get().to(models::list))
            .route("/subscriptions", web::post().to(models::create))
    })
    .bind("127.0.0.1:4000")?
    .run()
    .await
    
}
