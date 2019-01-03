// use actix_web::{http::Method, HttpRequest, middleware::{Middleware, Started, Response}, fs::NamedFile, server, Error, App, State};

// pub trait RequestIdentity {
//   fn identity(&self) -> Option<String>;

// }

// pub struct Auth;

// impl<S> Middleware<S> for Auth {
//   fn start(&self, req: &HttpRequest<S>) -> Result<Started, Error> {
//     Ok(Started::Done)
//   }
// }
