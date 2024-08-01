use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::{future::LocalBoxFuture, FutureExt};
use log::error;
use std::{
    env,
    future::{ready, Ready},
};

pub fn auth_middleware() {}

pub struct Authentication;
pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}
impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let api_key = req.headers().get("x-api-key");

        match api_key {
            None => {
                error!("No API_KEY provided");
                let http_res = HttpResponse::Unauthorized().body("Unauthorized");
                let (http_req, _) = req.into_parts();
                let res = ServiceResponse::new(http_req, http_res);
                return (async move { Ok(res.map_into_right_body()) }).boxed_local();
            }
            Some(user_api_key) => {
                if user_api_key.to_str().expect("Non ASCII values in api key")
                    != env::var("API_KEY").expect("No API_KEY env var provided")
                {
                    error!(
                        "Incorrect api key provided: {}",
                        user_api_key.to_str().unwrap()
                    );
                    let http_res = HttpResponse::Unauthorized().body("Unauthorized");
                    let (http_req, _) = req.into_parts();
                    let res = ServiceResponse::new(http_req, http_res);
                    return (async move { Ok(res.map_into_right_body()) }).boxed_local();
                }
            }
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}
