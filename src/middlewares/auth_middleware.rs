use std::borrow::Borrow;
use std::future::{ready, Ready};

use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage};
use actix_web::error::ErrorUnauthorized;
use actix_web::web::head;
use futures_util::future::LocalBoxFuture;
use crate::utils::auth::decode_token;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct AuthM;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for AuthM
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // println!("Hi from start. You requested: {}", req.borrow().path());
        if req.method() == Method::OPTIONS {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        let header =req.borrow().headers().get("Authorization");
        let header = match header {
            Some(header)=>{header},
            None=>{
                return Box::pin(async move{
                    return Err(ErrorUnauthorized("unauthorized"))
                })
            }
        };
        let header = header.to_str();
        let header = match header {
            Ok(header)=>{header}
            Err(err)=>{
                return Box::pin(async move{
                    return Err(ErrorUnauthorized("unauthorized"))
                })
            }
        };
        let claim = decode_token(header.to_string());
        let claim = match claim {
            Ok(claim)=>{claim},
            Err(err)=>{
                return Box::pin(async move{
                    return Err(ErrorUnauthorized("unauthorized"))
                })
            }
        };

        // add claim to req data
        req.borrow().extensions_mut().insert(claim);

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;


            // return Err(ErrorUnauthorized("unauthorized"));
            Ok(res)
        })
    }
}