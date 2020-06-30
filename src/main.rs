mod handlers;

use {
    std::cell::{RefCell, RefMut},
    std::ops::DerefMut,
    actix_web::{
        HttpServer, App, web, HttpRequest, HttpResponse, error,
        middleware::Logger,
        dev::{
            ServiceRequest
        }
    },
    actix_service::Service,
    futures::future::{ Future, ready, Ready },
    handlers::{
        simple_handler::*,
        extractor::*,
        error::*,
        middleware::*,
    }
};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    println!("Server is about to go up!");

    setup_logger();

    HttpServer::new(move || {
        App::new()
            // middleware logger
            // FnMut(ServiceRequest, &mut T::Service) -> R + Clone,
            .wrap_fn(|req, srv| {
                println!("verify token");
                let head = req.head();
                // head(&self) -> &RequestHead -> HeaderMap
                let user_auth = head.headers
                    // get<N>(&self, name: N) -> Option<&HeaderValue>
                    .get("Authorization")
                    // actix_web::http::header::HeaderValue -> Result<&str, Error>
                    .map(|header| header.to_str().ok().clone())
                    // .ok() // -> Option<&str>
                    .flatten()
                    .map(verify_token) // -> Option<UserAuth>
                    .unwrap();
                    // .unwrap_or(UserAuth::None);

                
            
                // save in request
                // extensions_mut(&self) -> RefMut<Extensions>
                head.extensions_mut().insert(user_auth);
                
                // call next service
                let fut = srv.call(req);
                async {
                    Ok(fut.await?)
                }
            })
            .wrap(Logger::default())
            .configure(config_app)
            .route("/hello", web::get().to(|| HttpResponse::Ok().body("hello, the server is up and running ~")))
    })
    .bind("127.0.0.1:8001")?
    .run()
    .await
}

fn config_app(cfg: &mut web::ServiceConfig) {
    cfg
    // .service(
    //     web::scope("/handler")
    //         .route("/text/{content}", web::get().to(text_handler))
    // )
    .service(
        web::scope("/out")
            .route("/json", web::get().to(out_json))
            .route("/custom/{content}", web::get().to(out_custom))
            .route("/text/{content}", web::get().to(out_text))
            .route("/stream", web::get().to(out_stream))
    )
    .service(
        web::scope("/in")
            .route("/qs", web::get().to(in_query))
            .route("/custom", web::post().to(in_custom))
            .route("/json", web::post().to(in_json))
            .route("/form", web::post().to(in_form))
    )
    .service(
        web::scope("/err")
            .route("/default", web::get().to(error_default))
            .service(
                web::scope("/custom")
                    .route("/internal", web::get().to(error_custom_internal))
                    .route("/mapped", web::get().to(error_custom_mapped))
                    .route("/userinfo", web::get().to(get_userinfo))
            )
    )
    ;
}

/*
App::Service
1. resource (web::resource(name))
    with routes of http actions as HEAD, GET, POST, DELETE ... sharing same resource url prefix like
    .route(web::get().to(view_fn))
    .route(web::post().to(view_fn))
2. scoped (web::scope(prefix))
    with further sub-urlprfixes like .route(sub_prefix, view_fn)
3. static files ...

App::Route
    like a root-scoped service
*/

fn setup_logger() {
    std::env::set_var("RUST_LOG", "debug,my_errors=debug,actix_web=info,simple_handlers=debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
}