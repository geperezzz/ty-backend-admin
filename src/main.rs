mod services;
mod models;

use actix_web::{
    HttpServer,
    App,
    middleware::{
        NormalizePath,
        TrailingSlash
    },
    web::Data
};
use actix_cors::Cors;
use anyhow::Context;
use env_logger::Env;
use sqlx::postgres::PgPoolOptions;
use tracing_actix_web::TracingLogger;

use services::*;

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    let _ = dotenvy::dotenv();
    let database_url = dotenvy::var("DATABASE_URL")
        .context("DATABASE_URL environment variable not found")?;
    let frontend_url = dotenvy::var("FRONTEND_URL")
        .context("FRONTEND_URL environment variable not found")?;

    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    let db = PgPoolOptions::new()
        .max_connections(6)
        .connect(database_url.as_str())
        .await
        .context("Couldn't connect to the database")?;
    let db = Data::new(db);


    HttpServer::new(move || {
        App::new()
            .app_data(db.clone())
            .wrap(TracingLogger::default())
            .wrap(NormalizePath::new(TrailingSlash::Always))
            .wrap(Cors::permissive().allowed_origin(frontend_url.as_str()))
            .configure(cities::configure)
    })
    .bind(("localhost", 8080))
    .context("Couldn't start the server")?
    .run()
    .await
    .context("Something failed during the server execution")
}
