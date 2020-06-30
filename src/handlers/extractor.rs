/*
actix provides 10 types of extractors (request parser, impl FromRequest)
1. Path     path variables, positional by Tuple or named style by providing custom type
2. Query    query string
3. Json
4. Form
5. Data     application state
6. HttpRequest  raw request
7. String   request payload as String
8. bytes::Bytes octet data
9. Payload  can be used to handle octet-stream
*/

use {
    std::{
        vec::Vec,
    },
    actix_web::{
        web, Error, Responder, HttpResponse, HttpRequest
    },
    futures::{
        future::{ Ready, ready },
    },
    serde::{ Serialize, Deserialize },
    serde_json::{
        json, Value, to_string, to_value
    },
    log::debug,
};

// #[derive()]
struct JsonResponse<T: Serialize>(T);

impl<T> Responder for JsonResponse<T> 
where T: Serialize{
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = to_string(&self.0).unwrap();

        // create response
        ready(Ok(
            HttpResponse::Ok().content_type("application/json").body(body)
        ))
    }
}

/*
Query is deserialize by serde_urlencoded crate
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct InQuery {
    name: String,
    id: u32,
}

pub async fn in_query(web::Query(qs): web::Query<InQuery>) -> impl Responder {
    debug!("in_query: {:?}", qs);

    JsonResponse(json!({
        "qs": to_value(qs).unwrap()
    }))
}

/*
Parse json
1. custom type
    only catches defined keys
2. serde_json::Value
    catches all
*/

/*
customer type
curl -iv -H 
"content-type: application/json" 
-d '{"name":"roger","id":32}' 
'http://localhost:8001/in/custom'
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct InJson {
    name: String,
    // make it optional, either absent or present with null 
    id: Option<u32>,
}

pub async fn in_custom(data: web::Json<InJson>) -> impl Responder {
    debug!("in json: {:?}", data);

    JsonResponse::<Value>(json!({
        "name": String::from("abc"),
        "id": 32,
    }))
}

/*
Deserialize to serde_json::Value type
curl -iv -H 
"content-type: application/json" 
-d '{"name":"roger","id":32}' 
'http://localhost:8001/in/json'

*/
pub async fn in_json(data: web::Json<Value>) -> impl Responder {
    debug!("in json: {:?}", data);

    JsonResponse::<Value>(json!({
        "name": String::from("abc"),
        "id": 32,
    }))
}

/*
form data
The handler will only get called if:
1. POST with content-type: application/x-www-form-urlencoded
2. Struct satisfy impl Deserialize
e.g
    curl -iv -H 
    "content-type: application/x-www-form-urlencoded" 
    -d "name=roger&id=32" 
    'http://localhost:8001/in/form'
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct InForm {
    name: String,
    id: u32,
}

pub async fn in_form(data: web::Form<InForm>) -> impl Responder {
    debug!("in form: {:?}", data);

    JsonResponse::<Value>(json!({
        "name": String::from("abc"),
        "id": 32,
    }))
}