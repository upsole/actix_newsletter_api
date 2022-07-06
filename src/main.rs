use actix_web::{web, App, HttpResponse, HttpServer};
use email_api::models;

async fn ok() -> HttpResponse {
    HttpResponse::Ok().body("Server is running.\n")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/ok", web::get().to(ok))
            .route("/subscriptions", web::get().to(models::list_accounts))
            .route("/subscriptions", web::post().to(models::post_account))
    })
    .bind("127.0.0.1:4000")?
    .run()
    .await
}
