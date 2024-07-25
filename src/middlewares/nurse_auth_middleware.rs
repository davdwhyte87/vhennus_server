// use std::borrow::Borrow;
// use std::future::{ready, Ready};
//
// use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage};
// use actix_web::error::ErrorUnauthorized;
// use actix_web::web::head;
// use futures_util::future::LocalBoxFuture;
// use crate::models::user::UserType;
// use crate::utils::auth::decode_token;
//
// // There are two steps in middleware processing.
// // 1. Middleware initialization, middleware factory gets called with
// //    next service in chain as parameter.
// // 2. Middleware's call method gets called with normal request.
// pub struct NurseAuth;
//
// // Middleware factory is `Transform` trait
// // `S` - type of the next service
// // `B` - type of response's body
// impl<S, B> Transform<S, ServiceRequest> for NurseAuth
//     where
//         S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
//         S::Future: 'static,
//         B: 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type InitError = ();
//     type Transform = NurseAuthMiddleware<S>;
//     type Future = Ready<Result<Self::Transform, Self::InitError>>;
//
//     fn new_transform(&self, service: S) -> Self::Future {
//         ready(Ok(NurseAuthMiddleware { service }))
//     }
// }
//
// pub struct NurseAuthMiddleware<S> {
//     service: S,
// }
//
// impl<S, B> Service<ServiceRequest> for NurseAuthMiddleware<S>
//     where
//         S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
//         S::Future: 'static,
//         B: 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
//
//     forward_ready!(service);
//
//     fn call(&self, req: ServiceRequest) -> Self::Future {
//         // println!("Hi from start. You requested: {}", req.borrow().path());
//
//
//         let header =req.borrow().headers().get("Authorization");
//         let header = match header {
//             Some(header)=>{header},
//             None=>{
//                 return Box::pin(async move{
//                     return Err(ErrorUnauthorized("unauthorized"))
//                 })
//             }
//         };
//         let header = header.to_str();
//         let header = match header {
//             Ok(header)=>{header}
//             Err(err)=>{
//                 return Box::pin(async move{
//                     return Err(ErrorUnauthorized("unauthorized"))
//                 })
//             }
//         };
//         let claim = decode_token(header.to_string());
//         let claim = match claim {
//             Ok(claim)=>{claim},
//             Err(err)=>{
//                 return Box::pin(async move{
//                     return Err(ErrorUnauthorized("unauthorized"))
//                 })
//             }
//         };
//
//         if !(claim.role == UserType::Nurse || claim.role == UserType::Hospital){
//             return Box::pin(async move{
//                 return Err(ErrorUnauthorized("You do not have permission"))
//             })
//         }
//
//         // add claim to req data
//         req.borrow().extensions_mut().insert(claim);
//
//         let fut = self.service.call(req);
//         Box::pin(async move {
//             let res = fut.await?;
//
//
//             // return Err(ErrorUnauthorized("unauthorized"));
//             Ok(res)
//         })
//     }
// }