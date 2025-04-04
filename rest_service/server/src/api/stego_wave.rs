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

macro_rules! parse_form {
    ($payload:expr) => {
        match parse_multipart_payload($payload).await {
            Ok(data) => data,
            Err(err) => return HttpResponse::InternalServerError().body(err),
        }
    };
}

macro_rules! get_stego {
    ($format:expr, $lsb_deep:expr, $settings:expr) => {
        match get_stego_by_str(&$format, $lsb_deep as _, (*$settings.into_inner()).clone()) {
            Ok(stego) => stego,
            Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
        }
    };
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
    let multipart = parse_form!(payload);
    let hide_request: HideRequest = match multipart.try_into() {
        Ok(req) => req,
        Err(err) => return HttpResponse::BadRequest().body(err),
    };

    let stego = get_stego!(hide_request.format, hide_request.lsb_deep, settings);

    let (mut samples, spec) = match stego.read_samples_from_byte(hide_request.file) {
        Ok(data) => data,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    if let Err(err) =
        stego.hide_message_binary(&mut samples, hide_request.message, hide_request.password)
    {
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
    let multipart = parse_form!(payload);
    let extract_request: ExtractRequest = match multipart.try_into() {
        Ok(req) => req,
        Err(err) => return HttpResponse::BadRequest().body(err),
    };

    let stego = get_stego!(extract_request.format, extract_request.lsb_deep, settings);

    let (samples, _) = match stego.read_samples_from_byte(extract_request.file) {
        Ok(data) => data,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    match stego.extract_message_binary(&samples, extract_request.password) {
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
    let multipart = parse_form!(payload);
    let clear_request: ClearRequest = match multipart.try_into() {
        Ok(req) => req,
        Err(err) => return HttpResponse::BadRequest().body(err),
    };

    let stego = get_stego!(clear_request.format, clear_request.lsb_deep, settings);

    let (mut samples, spec) = match stego.read_samples_from_byte(clear_request.file) {
        Ok(data) => data,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    match stego.clear_secret_message_binary(&mut samples, clear_request.password) {
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
