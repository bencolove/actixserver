/*
Middlewares are typically involved in:
1. Pre-process the Request
2. Post-process a Response
3. Modify application state
4. Access external services (redis, logging, sessions)

Middlewares can be registered (effects on) `App`, `scope`, `Resource` and
executed in reversed order as they are declared.

A `middleware` is a type (struct) that implements
1. Service trait, `actix_web::dev::Service`
2. Transfrom trait, `actix_web::dev::Transform`
each method should return a `Future`


trait Service {
    type Request;
    type Response;
    type Error;
    type Future: Future;

    // poll_ready the next service down in the chain
    fn poll_ready(&mut self, ctx: &mut Context) -> Poll<Result<(), Self::Error>>;

    // do your job and call next servcie down in the chain
    fn call(&mut self, req: Self::Request) -> Self::Future;
}

pub trait Transform<S> 
where
    <Self::Transform as Service>::Request == Self::Request,
    <Self::Transform as Service>::Response == Self::Response,
    <Self::Transform as Service>::Error == Self::Error,
    <Self::Future as Future>::Output == Result<Self::Transform, Self::InitError>, 
{
    type Request;
    type Response;
    type Error;
    type Transform: Service;
    type InitError;
    type Future: Future;

    fn new_transform(&self, service: S) -> Self::Future;

    fn map_init_err<F, E>(self, f: F) -> TransformMapInitErr<Self, S, F, E>
    where
        F: Fn(Self::InitError) -> E + Clone,
    { ... }
}

There are two steps in middleware processing.
1. Middleware initialization, middleware factory gets called with
   next service in chain as parameter. 
   -> By Transform::new_transform() with next `Service` in the chain
2. Middleware's call method gets called with normal request.
    -> Invoked by previous middleware up in the chain
*/
use {
    std::time::Duration,
    tokio::time::{
        delay_for,
    },

    std::{
        rc::Rc,
        pin::Pin,
        task::{Context, Poll},
    },
    actix_service::{Service, Transform},
    actix_web::{
        dev::{ServiceRequest, ServiceResponse},
        Responder,
        Error,
    },
    futures::future::{
        Future, ok, err, Ready
    },
};

#[derive(Debug)]
pub enum UserAuth {
    None,
    UserInfo {
        name: String,
        token: String,
    },
}

pub fn verify_token(token: &str) -> UserAuth {
    
    // delay_for(Duration::from_secs(1)).await;

    if (token.is_empty()) {
        println!("missing token");
    } else {
        println!("token is {}", token);
    }

    UserAuth::UserInfo {
        name: String::from("roger"),
        token: String::from(token),
    }
}

/*
In order to implement a middleware, thus are needed:
1. a struct for construction impl Transfrom
2. a struct for invocation imple Service
*/

pub struct BearerTokenMiddlewareInit;

pub struct BearerTokenMiddleware<Service> {
    // keep refence to next Service
    next_service: Service,
}
/*
S next service type
B response's body type
*/
impl<S, B> Transform<S> for BearerTokenMiddlewareInit 
where
    S: Service<
        Request = ServiceRequest,
        Response = ServiceResponse<B>,
        Error = Error 
    > + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = BearerTokenMiddleware<S>;
    type InitError = ();
    type Future: = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // construct
        ok( BearerTokenMiddleware { next_service: service }) 
    }
}

impl <S, B> Service for BearerTokenMiddleware<S> 
where
    S: Service<
        Request = ServiceRequest,
        Response = ServiceResponse<B>,
        Error = Error 
    > + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<(dyn Future<Output = Result<Self::Response, Self::Error>> + 'static)>>;

    // poll next service
    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.next_service.poll_ready(cx)
    }

    // request moved in !!
    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        println!("BearerTokenMiddleware begin for path: {}", req.path());
        
        // dont know how to go round async block lifetime matter in order to do it async
        let pre_proess_rst = pre_process_request(&req);
        if let Ok(Some(fast_resp)) = pre_proess_rst {
            return Box::pin(ok(fast_resp));
        } else if let Err(error) = pre_proess_rst {
            return Box::pin(err(error));
        }
        // only Ok(None) means to process

        // can not use `self` in asyc block due to lifetime matter
        let fut = self.next_service.call(req);

        // do your stuff pre-process request
        Box::pin( async move {
            // dont know how to asynchronously handle pre_process(req)
            // let pre_rst = pre_process_request(req).await?;
            
            // let mut next_service = self.next_service;
        // forward to next service, request move along !!

        // why pin ?? threaded ??
        // Box::pin shorthand for Pin::new(Box::new(T))
         
            // error bubble up
            let resp = fut.await?;

            // do your stuff post-process response
            let post_rst = post_process_response(resp).await;
            println!("BearerTokenMiddleware finish");
            post_rst
        })
    }
}

/*
Result<Option<Response>, Error>
if:
1. Error: shortcircuit to exit with Error
2. Option<Response> = None: proceed to next service
2. Option<Response> = Some<Response>: shortcircuit to response
*/
fn pre_process_request<B>(request: &ServiceRequest) -> Result<Option<ServiceResponse<B>>, Error> {
    println!("pre_process_request");

    let head = request.head();
    // head(&self) -> &RequestHead -> HeaderMap
    let user_auth = head.headers
        // get<N>(&self, name: N) -> Option<&HeaderValue>
        .get("Authorization")
        // actix_web::http::header::HeaderValue -> Result<&str, Error>
        .map(|header| header.to_str().ok().clone())
        // .ok() // -> Option<&str>
        .flatten()
        .map(verify_user_by_bearertoken) // -> Option<UserAuth>
        ;
    // handle result
    if let Some(auth) = user_auth {
        // token present
        println!("auth: {:?}", auth);
        // insert auth result into request.extensions
    } else {
        // token missing
        println!("token is missing");
    }


    Ok(None)
}

async fn post_process_response<B>(response: ServiceResponse<B>) -> Result<ServiceResponse<B>, Error> {
    println!("post_process_response");
    Ok(response)
}

fn verify_user_by_bearertoken (token: &str) -> UserAuth {

    if token.is_empty() {
        println!("token is empty");
        return UserAuth::None;
    }

    if token != "abctoken" {
        println!("token is {}", token);
        return UserAuth::None;
    }

    UserAuth::UserInfo {
        name: String::from("roger"),
        token: String::from(token),
    }
}

