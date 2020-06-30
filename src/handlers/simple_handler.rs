use {
    futures::{
        future::{ ok, Ready, ready },
        stream::once,
    },
    actix_web::{
        App, web, get, HttpRequest, HttpResponse, Responder, Error
    },
    serde::{
        Serialize
    },
    serde_json::{
        json, Value, to_string, from_str
    },
    bytes,
    log::debug,
};

pub async fn out_text(data: web::Path<(String,)>) -> impl Responder {
    HttpResponse::Ok().body(
        format!("text data: {}", data.0)
    )
}

pub async fn out_json() -> impl Responder {
    debug!("out_json");
    web::Json(json!({
        "name": "roger",
        "age": 39,
        "favourite_num": [1, 3, 17]
    }))
    .with_header("messasge", "out json")
}

/*
response with custom eventually converted to text (unicode bytes) 
*/
#[derive(Serialize)]
pub struct CustomOut {
    name: String,
    // never serialize for privacy
    #[serde(skip_serializing)]
    age: u32,
    // conditional serialize
    #[serde(skip_serializing_if = "std::string::String::is_empty")]
    message: String,
}

impl Responder for CustomOut {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = to_string(&self).unwrap();

        // create response
        ready(Ok(
            HttpResponse::Ok().content_type("application/json").body(body)
        ))
    }
}

pub async fn out_custom(data: web::Path<(Option<String>,)>) -> impl Responder {

    debug!("{:?}", data.0.as_ref());

    CustomOut {
        name: "Roger".to_owned(),
        age: 38,
        message: data.0.as_ref().unwrap_or(&"".to_string().to_owned()).to_string(),
    }
}

/*
stream response
*/
pub async fn out_stream() -> impl Responder {
    let body = once(ok::<_, Error>(bytes::Bytes::from_static(b"stream content")));

    HttpResponse::Ok().content_type("application/json").streaming(body)
}