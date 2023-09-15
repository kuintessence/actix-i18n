use std::collections::HashMap;
use std::io;

use actix_i18n::{I18NResources, Locale};
use actix_web::middleware::Logger;
use actix_web::web::Path;
use actix_web::App;
use actix_web::{get, HttpServer};

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    HttpServer::new(move || {
        let resources = I18NResources::builder()
            .add_path("resources")
            .use_isolating(false)
            .build()
            .unwrap();

        App::new()
            .wrap(Logger::default())
            .app_data(resources)
            .service(index)
            .service(welcome_tuple)
            .service(welcome_hashmap)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

#[get("/")]
async fn index(locale: Locale) -> String {
    locale
        .text("hello-world")
        .unwrap_or_else(|_| "error".to_string())
}

#[get("/welcome_tuple/{name}")]
async fn welcome_tuple(locale: Locale, name: Path<String>) -> String {
    locale
        .text_with_args("welcome", (("name", name.into_inner()),))
        .unwrap_or_else(|_| "error".to_string())
}

#[get("/welcome_hashmap/{name}")]
async fn welcome_hashmap(locale: Locale, name: Path<String>) -> String {
    let mut args = HashMap::new();
    args.insert("name", name.into_inner());

    locale
        .text_with_args("welcome", args)
        .unwrap_or_else(|_| "error".to_string())
}
