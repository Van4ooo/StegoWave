use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use std::net::TcpListener;
use stego_wave::configuration::Settings;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn run_server(
    listener: TcpListener,
    stego_wave_lib: Settings,
) -> Result<Server, std::io::Error> {
    let settings = web::Data::new(stego_wave_lib);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .configure(super::api::stego_wave::routers)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api/openapi.json", crate::api::api_docs::ApiDoc::openapi()),
            )
            .app_data(settings.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
