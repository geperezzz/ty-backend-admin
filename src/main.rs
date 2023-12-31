mod models;
mod services;
mod utils;
mod views;

use actix_cors::Cors;
use actix_web::{
    middleware::{NormalizePath, TrailingSlash},
    web::{self, Data},
    App, HttpServer,
};
use anyhow::Context;
use env_logger::Env;
use sqlx::postgres::PgPoolOptions;
use tracing_actix_web::TracingLogger;

use services::*;

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().context(".env not found")?;

    let database_url =
        dotenvy::var("DATABASE_URL").context("DATABASE_URL environment variable not found")?;
    let frontend_url =
        dotenvy::var("FRONTEND_URL").context("FRONTEND_URL environment variable not found")?;

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
            .configure(clients::configure)
            .configure(vehicles::configure)
            .configure(states::configure)
            .configure(vehicle_models::configure)
            .configure(roles::configure)
            .configure(supply_lines::configure)
            .service(web::scope("/products").configure(products::configure))
            .service(web::scope("/staff").configure(staff::configure))
            .service(web::scope("/activities").configure(activities::configure))
            .service(web::scope("/dealerships").configure(dealerships::configure))
            .service(
                web::scope("/least-employed-employees")
                    .configure(least_employed_employees::configure),
            )
            .service(
                web::scope("/least-profitable-dealerships")
                    .configure(least_profitable_dealerships::configure),
            )
            .service(
                web::scope("/least-requested-services")
                    .configure(least_requested_services::configure),
            )
            .service(web::scope("/least-used-products").configure(least_used_products::configure))
            .service(
                web::scope("/maintenance-schedules").configure(maintenance_schedules::configure),
            )
            .service(
                web::scope("/most-attended-vehicle-models")
                    .configure(most_attended_vehicle_models::configure),
            )
            .service(
                web::scope("/most-employed-employees")
                    .configure(most_employed_employees::configure),
            )
            .service(
                web::scope("/most-profitable-dealerships")
                    .configure(most_profitable_dealerships::configure),
            )
            .service(
                web::scope("/most-requested-services")
                    .configure(most_requested_services::configure),
            )
            .service(web::scope("/most-used-products").configure(most_used_products::configure))
            .service(web::scope("/no-show-clients").configure(no_show_clients::configure))
            .service(
                web::scope("/vehicle-applied-services")
                    .configure(vehicle_applied_services::configure),
            )
            .service(web::scope("/services").configure(services::services::configure))
            .service(
                web::scope("/activities-prices").configure(services::activities_prices::configure),
            )
            .service(web::scope("/discounts").configure(services::discounts::configure))
            .service(web::scope("/invoices").configure(services::invoices::configure))
            .service(web::scope("/payments").configure(services::payments::configure))
            .service(web::scope("/stock").configure(stock::configure))
    })
    .bind(("localhost", 8080))
    .context("Couldn't start the server")?
    .run()
    .await
    .context("Something failed during the server execution")
}
