use std::borrow::Borrow;
use std::future::{ready, Ready};
use actix_web::http::Method;
use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage};
use actix_web::error::ErrorUnauthorized;
use futures_util::future::LocalBoxFuture;
use crate::utils::auth::decode_token;

pub struct AuthM;

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
        // Allow OPTIONS requests to pass through without authentication (for CORS preflight)
        if req.method() == Method::OPTIONS {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        let header = req.borrow().headers().get("Authorization");
        let header = match header {
            Some(header) => header,
            None => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("unauthorized"))
                })
            }
        };
        
        let header_str = match header.to_str() {
            Ok(header) => header,
            Err(_) => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("unauthorized"))
                })
            }
        };
        
        let claim = decode_token(header_str.to_string());
        let claim = match claim {
            Ok(claim) => claim,
            Err(_) => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("unauthorized"))
                })
            }
        };

        // Add claim to req extensions
        req.borrow().extensions_mut().insert(claim);

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}