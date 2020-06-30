/*
Deal with
1. `actix_web::error::Error`
2. `actix_web::error::ResponseError`

actix will by default render std::Result<Value, E:ref ResponseError> by coercing it into `Responder`

actix_web::error::ResponseError
pub trait ResponseError {
    fn error_response(&self) -> HttpResponse;
    fn status_code(&self) -> StatusCode;
}

std::error::Error
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> { ... }
    fn backtrace(&self) -> Option<&Backtrace> { ... }
    fn description(&self) -> &str { ... }
    fn cause(&self) -> Option<&dyn Error> { ... }
}
fmt::Display and Error::source(&self) should be implemented.


Mapping from remote caret's Errors to ResponseError::HttpInternalServerError

Mapping custom Error:
    impl actix_web::error for CustomError {}
using default implementation

*/
use {
    std::{
        vec::Vec,
        string::String,
    },
    actix_web::{
        web, error, Responder, HttpResponse, HttpRequest, http
    },
    actix_http::ResponseBuilder,
    futures::{
        future::{ Ready, ready },
    },
    serde::{ Serialize, Deserialize },
    serde_json::{
        json, Value, to_string, to_value
    },
    failure::Fail,
    log::debug,
    super::middleware::*,
};

/*
Default implementation of ResponseError will be render to
'default error', 500
*/
#[derive(Fail, Debug)]
#[fail(display = "default error")]
pub struct DefaultError {
    message: &'static str,
}

impl error::ResponseError for DefaultError {}

pub async fn error_default() -> Result<String, DefaultError> {
    Err(DefaultError{
        message: "do you see this error",
    })
}

/*
Build more meaningful errors
*/
#[derive(Debug, Fail)]
pub enum SysErr {
    #[fail(display = "internal error")]
    IntenalError,
    #[fail(display = "bad request")]
    BadRequestError(&'static str),
}

impl error::ResponseError for SysErr {
    fn error_response(&self) -> HttpResponse {
        // HttpResponse::Ok() variant?
        ResponseBuilder::new(self.status_code())
            .set_header(http::header::CONTENT_TYPE, "applicaion/json; charset=utf-8")
            // .body(self.to_string())
            .body(
                json!({
                    "success": false,
                    "message": self.to_string(),
                })
            )
    }
    fn status_code(&self) -> http::StatusCode {
        match *self {
            SysErr::IntenalError => http::StatusCode::INTERNAL_SERVER_ERROR,
            SysErr::BadRequestError(..) => http::StatusCode::BAD_REQUEST,
        }
    }
}

pub async fn error_custom_internal() -> Result<String, SysErr> {
    Err(SysErr::IntenalError)
}

/*
use helper to map from custom error to predefined ResponseError
*/

pub trait Messagible {
    fn message(&self) -> &'static str;
}

impl Messagible for SysErr {
    fn message(&self) -> &'static str {
        match *self {
            SysErr::IntenalError => "internal error",
            SysErr::BadRequestError(message) => message, 
        }
    }
}

pub async fn error_custom_mapped() -> actix_web::Result<String> {
    let result = Err(SysErr::BadRequestError("date"));

    // result.as_ref().map_err(|e| error::ErrorBadRequest(e.message()))

    result.map_err(|e| error::ErrorBadRequest(e.message()))?
}

pub async fn get_userinfo(req: HttpRequest) -> actix_web::Result<String> {
    // Ok(match req.extensions().get::<UserAuth>() {
    //     Some(info) => format!("user: {}", info.name),
    //     None => "no user info".to_owned(),
    // })

    Ok(if let Some(auth) = req.extensions().get::<UserAuth>() {
        println!("{:?}", auth);
        if let UserAuth::UserInfo{ name: name, .. } = auth {
            format!("user: {}", name)
        } else {
            "no user info".to_owned()
        }
    } else {
        "no user info".to_owned()
    })
}

