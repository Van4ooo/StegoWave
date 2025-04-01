use utoipa::OpenApi;

use crate::api;
use crate::models;

#[derive(OpenApi)]
#[openapi(
    paths(
        api::stego_wave::hide_message,
        api::stego_wave::extract_message,
        api::stego_wave::clear_message,
    ),
    components(
        schemas(
            models::request_object::HideRequest,
            models::request_object::ExtractRequest,
            models::request_object::ClearRequest
        )
    ),
    tags(
        (name = "StegoWave"),
    )
)]
pub struct ApiDoc;
