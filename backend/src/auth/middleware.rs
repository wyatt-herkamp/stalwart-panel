use crate::auth::session::{Session, SessionManager};
use crate::auth::AuthenticationRaw;
use actix_service::{forward_ready, Service, Transform};
use actix_web::body::{BoxBody, EitherBody};
use actix_web::cookie::Cookie;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::{header, Method};
use actix_web::web::Data;
use actix_web::{Error, HttpMessage, HttpResponse};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;
use tracing::log::warn;

pub struct HandleSession(pub Data<SessionManager>);

impl<S, B> Transform<S, ServiceRequest> for HandleSession
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Transform = SessionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionMiddleware {
            service: Rc::new(service),
            session_manager: self.0.clone(),
        }))
    }
}
pub struct SessionMiddleware<S> {
    service: Rc<S>,
    session_manager: Data<SessionManager>,
}

impl<S, B> SessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    async fn handle_session(
        session_manager: Data<SessionManager>,
        req: &ServiceRequest,
        cookie: Cookie<'static>,
    ) -> Result<(), HttpResponse> {
        let session: Option<Session> = match session_manager.get_session(cookie.value()) {
            Ok(ok) => ok,
            Err(e) => {
                warn!("Session Manager Error: {}", e);
                return Err(HttpResponse::InternalServerError().body("Session Manager Error"));
            }
        };
        if let Some(session) = session {
            let raw = AuthenticationRaw::Session(session);
            req.extensions_mut().insert(raw);
        }
        Ok(())
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `service`: A clone of the service
    /// * `req`: The request
    /// * `session_manager`:  The session manager
    ///
    /// returns: Result<ServiceResponse<EitherBody<B, BoxBody>>, Error>
    ///    - Ok: The response  - Will just be the call to the next handler
    ///   - Err: The error - Will be an error response
    async fn handle_authentication(
        service: Rc<S>,
        req: ServiceRequest,
        session_manager: Data<SessionManager>,
    ) -> Result<ServiceResponse<EitherBody<B, BoxBody>>, Error> {
        if let Some(_auth) = req.headers().get(header::AUTHORIZATION) {
            todo!("Handle API Tokens")
        } else if let Some(cookie) = req.cookie("session") {
            if let Err(e) = Self::handle_session(session_manager, &req, cookie).await {
                return Ok(req.into_response(e.map_into_right_body()));
            }
        }
        let fut = service.call(req);

        let res = fut.await?;
        Ok(res.map_into_left_body())
    }
}
impl<S, B> Service<ServiceRequest> for SessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Check if its an OPTIONS request. If so exit early and let the request pass through
        if req.method() == Method::OPTIONS {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }
        // Grab required data from the service
        let session_manager = self.session_manager.clone();
        // Move into an async block
        let session = Self::handle_authentication(self.service.clone(), req, session_manager);
        Box::pin(async move {
            let res = session.await?;
            Ok(res)
        })
    }
}
