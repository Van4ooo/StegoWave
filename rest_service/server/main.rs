use actix_web::{App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod api;
mod models;
mod services;
mod tracing_config;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_config::conf_logger();

    let app = move || {
        App::new().configure(api::stego_wave::routers).service(
            SwaggerUi::new("/swagger-ui/{_:.*}")
                .url("/api/openapi.json", api::api_docs::ApiDoc::openapi()),
        )
    };

    HttpServer::new(app).bind(("0.0.0.0", 8080))?.run().await?;

    Ok(())
}
