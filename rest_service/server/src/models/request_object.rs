use serde::Deserialize;
use utoipa::ToSchema;

#[allow(unused)]
#[derive(Deserialize, ToSchema)]
pub struct HideRequest {
    #[schema(value_type = String, format = "binary")]
    pub file: Vec<u8>,
    #[schema(example = "Secret message")]
    pub message: String,
    #[schema(example = "qwerty1234")]
    pub password: String,
    #[schema(example = "wav16")]
    pub format: String,
    #[schema(example = "1", minimum = 1)]
    pub lsb_deep: u8,
}

#[allow(unused)]
#[derive(Deserialize, ToSchema)]
pub struct ExtractRequest {
    #[schema(value_type = String, format = "binary")]
    pub file: Vec<u8>,
    #[schema(example = "qwerty1234")]
    pub password: String,
    #[schema(example = "wav16")]
    pub format: String,
    #[schema(example = "1", minimum = 1)]
    pub lsb_deep: u8,
}

#[allow(unused)]
#[derive(Deserialize, ToSchema)]
pub struct ClearRequest {
    #[schema(value_type = String, format = "binary")]
    pub file: Vec<u8>,
    #[schema(example = "qwerty1234")]
    pub password: String,
    #[schema(example = "wav16")]
    pub format: String,
    #[schema(example = "1", minimum = 1)]
    pub lsb_deep: u8,
}
