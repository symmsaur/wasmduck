use actix_files::{Files, NamedFile};
use actix_web::{web, App, HttpServer, Result};

async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/index.html", web::get().to(index))
            .service(Files::new("/", "target/wasm32-unknown-unknown/release"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
