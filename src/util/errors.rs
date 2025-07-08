use reqwest::StatusCode;
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ApiError {
    RequestError,        // Error occurred while making the request
    NoRefreshToken,      // No refresh token was found
    ResponseParseError,  // Error occurred while parsing the response
    ResponseDataError,   // Missing or invalid data in the response
    NoAccessToken,       // No access token was found
    InvalidAccessToken,  // Invalid access token
    BackoffError,        // Error occurred while backing off
    BrowserError,        // Error occurred while interacting with browser
    InternalServerError, // Error occurred on the api server

    ResponseError204, // Error returned in the response
    ResponseError401, // Error returned in the response
    ResponseError403, // Error returned in the response
    ResponseError404, // Error returned in the response
    ResponseError429, // Error returned in the response
    ResponseError500, // Error returned in the response
    ResponseError502, // Error returned in the response
    ResponseError503, // Error returned in the response
    ResponseError504, // Error returned in the response
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", return_error_message(self))
    }
}

pub fn return_response_error(sc: StatusCode) -> ApiError {
    match sc {
        StatusCode::NO_CONTENT => ApiError::ResponseError204,
        StatusCode::UNAUTHORIZED => ApiError::ResponseError401,
        StatusCode::FORBIDDEN => ApiError::ResponseError403,
        StatusCode::NOT_FOUND => ApiError::ResponseError404,
        StatusCode::TOO_MANY_REQUESTS => ApiError::ResponseError429,
        StatusCode::INTERNAL_SERVER_ERROR => ApiError::ResponseError500,
        StatusCode::BAD_GATEWAY => ApiError::ResponseError502,
        StatusCode::SERVICE_UNAVAILABLE => ApiError::ResponseError503,
        StatusCode::GATEWAY_TIMEOUT => ApiError::ResponseError504,
        _ => ApiError::InternalServerError,
    }
}

pub fn return_response_code(ae: ApiError) -> StatusCode {
    match ae {
        ApiError::ResponseError204 => StatusCode::NO_CONTENT,
        ApiError::ResponseError401 => StatusCode::UNAUTHORIZED,
        ApiError::ResponseError403 => StatusCode::FORBIDDEN,
        ApiError::ResponseError404 => StatusCode::NOT_FOUND,
        ApiError::ResponseError429 => StatusCode::TOO_MANY_REQUESTS,
        ApiError::ResponseError500 => StatusCode::INTERNAL_SERVER_ERROR,
        ApiError::ResponseError502 => StatusCode::BAD_GATEWAY,
        ApiError::ResponseError503 => StatusCode::SERVICE_UNAVAILABLE,
        ApiError::ResponseError504 => StatusCode::GATEWAY_TIMEOUT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub fn return_error_message(e: &ApiError) -> String {
    match e {
        ApiError::RequestError => "Error occurred while making the request".to_string(),
        ApiError::NoRefreshToken => "No refresh token was found".to_string(),
        ApiError::ResponseParseError => "Error occurred while parsing the response".to_string(),
        ApiError::ResponseDataError => "Missing or invalid data in the response".to_string(),
        ApiError::NoAccessToken => "No access token was found".to_string(),
        ApiError::InvalidAccessToken => "Invalid access token".to_string(),
        ApiError::BackoffError => "Error occurred while backing off".to_string(),
        ApiError::BrowserError => "Error occurred while interacting with browser".to_string(),
        ApiError::InternalServerError => "Error occurred on the api server".to_string(),

        ApiError::ResponseError204 => "No content returned in the response".to_string(),
        ApiError::ResponseError401 => "Unauthorized request".to_string(),
        ApiError::ResponseError403 => "Forbidden request".to_string(),
        ApiError::ResponseError404 => "Resource not found".to_string(),
        ApiError::ResponseError429 => "Too many requests".to_string(),
        ApiError::ResponseError500 => "Internal server error".to_string(),
        ApiError::ResponseError502 => "Bad gateway".to_string(),
        ApiError::ResponseError503 => "Service unavailable".to_string(),
        ApiError::ResponseError504 => "Gateway timeout".to_string(),
    }
}
