use actix_multipart::Multipart;
use actix_web::{HttpResponse, Responder, post, web};
use stego_wave::AudioSteganography;
use stego_wave::configuration::StegoWaveLib;
use stego_wave::error::StegoError;
use stego_wave::formats::get_stego_by_str;

use crate::models::request_object::{ClearRequest, ExtractRequest, HideRequest};
use crate::services::stego_wave::parse_multipart_payload;

macro_rules! audio_response_from_samples {
    ($stego:expr, $spec:expr, $samples:expr) => {{
        let out_buf = match $stego.write_samples_to_byte($spec, &$samples) {
            Ok(buf) => buf,
            Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        };

        HttpResponse::Ok()
            .append_header(("Accept-Ranges", "bytes"))
            .append_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", $stego.default_filename()),
            ))
            .content_type("audio/wav")
            .body(out_buf)
    }};
}

macro_rules! get_required_field {
    ($payload:expr, $file_bytes:expr, $message:expr, $password:expr, $format:expr) => {{
        let mut multipart = match parse_multipart_payload($payload).await {
            Ok(data) => data,
            Err(e) => return HttpResponse::InternalServerError().body(e),
        };

        match multipart.get_required_field($file_bytes, $message, $password, $format) {
            Ok(data) => data,
            Err(err) => return HttpResponse::BadRequest().body(err),
        }
    }};
}

#[utoipa::path(
    post,
    tag = "StegoWave",
    path = "/api/hide_message",
    request_body (
        content = HideRequest,
        content_type = "multipart/form-data"
    ),
    responses (
        (status = 200, description = "Returns an audio file with a hidden message", content_type = "audio/wav"),
        (status = 400, description = "Bad request, missing required fields", content_type = "text/plain"),
        (status = 500, description = "Internal server error", content_type = "text/plain")
    )
)]
#[post("/api/hide_message")]
pub async fn hide_message(payload: Multipart, settings: web::Data<StegoWaveLib>) -> impl Responder {
    let (file_bytes, message, password, format, lsb_deep) =
        get_required_field!(payload, true, true, true, true);

    let stego = match get_stego_by_str(&format, lsb_deep, (*settings.into_inner()).clone()) {
        Ok(stego) => stego,
        Err(err) => return HttpResponse::BadRequest().body(err),
    };

    let (mut samples, spec) = match stego.read_samples_from_byte(file_bytes) {
        Ok(data) => data,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    if let Err(err) = stego.hide_message_binary(&mut samples, &message, &password) {
        return HttpResponse::InternalServerError().body(err.to_string());
    }

    audio_response_from_samples!(stego, spec, samples)
}

#[utoipa::path(
    post,
    tag = "StegoWave",
    path = "/api/extract_message",
    request_body (
        content = ExtractRequest,
        content_type = "multipart/form-data"
    ),
    responses (
        (status = 200, description = "Returns the extracted message", content_type = "text/plain"),
        (status = 400, description = "Bad request, missing required fields", content_type = "text/plain"),
        (status = 500, description = "Internal server error", content_type = "text/plain")
    )
)]
#[post("/api/extract_message")]
pub async fn extract_message(
    payload: Multipart,
    settings: web::Data<StegoWaveLib>,
) -> impl Responder {
    let (file_bytes, _, password, format, lsb_deep) =
        get_required_field!(payload, true, false, true, true);

    let stego = match get_stego_by_str(&format, lsb_deep, (*settings.into_inner()).clone()) {
        Ok(stego) => stego,
        Err(err) => return HttpResponse::BadRequest().body(err),
    };

    let (samples, _) = match stego.read_samples_from_byte(file_bytes) {
        Ok(data) => data,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    match stego.extract_message_binary(&samples, &password) {
        Ok(msg) => HttpResponse::Ok().body(msg),
        Err(StegoError::IncorrectPassword) => {
            HttpResponse::BadRequest().body(StegoError::IncorrectPassword.to_string())
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[utoipa::path(
    post,
    tag = "StegoWave",
    path = "/api/clear_message",
    request_body (
        content = ClearRequest,
        content_type = "multipart/form-data"
    ),
    responses (
        (status = 200, description = "Returns the audio file with the cleared message", content_type = "audio/wav"),
        (status = 400, description = "Bad request, missing required fields", content_type = "text/plain"),
        (status = 500, description = "Internal server error", content_type = "text/plain")
    )
)]
#[post("/api/clear_message")]
pub async fn clear_message(
    payload: Multipart,
    settings: web::Data<StegoWaveLib>,
) -> impl Responder {
    let (file_bytes, _, password, format, lsb_deep) =
        get_required_field!(payload, true, false, true, true);

    let stego = match get_stego_by_str(&format, lsb_deep, (*settings.into_inner()).clone()) {
        Ok(stego) => stego,
        Err(err) => return HttpResponse::BadRequest().body(err),
    };

    let (mut samples, spec) = match stego.read_samples_from_byte(file_bytes) {
        Ok(data) => data,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    match stego.clear_secret_message_binary(&mut samples, &password) {
        Ok(()) => {}
        Err(StegoError::IncorrectPassword) => {
            return HttpResponse::BadRequest().body(StegoError::IncorrectPassword.to_string());
        }
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    }

    audio_response_from_samples!(stego, spec, samples)
}

pub fn routers(cfg: &mut web::ServiceConfig) {
    cfg.service(hide_message)
        .service(extract_message)
        .service(clear_message);
}
