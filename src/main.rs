#[macro_use]
extern crate serde_derive;

extern crate actix_web;
extern crate listenfd;
extern crate reqwest;
extern crate url;
extern crate hyper;
extern crate base64;

mod oauth;

use listenfd::ListenFd;
use actix_web::{http::Method, server, App, HttpRequest};

fn index(_req: &HttpRequest) -> &'static str {
    "Hello world!"
}

fn main() {
    let mut listenfd = ListenFd::from_env();
    let mut server = server::new(|| {
        App::new()
            .resource("/", |r| r.f(index))
            .resource("/oauth/{service}/start", |r| r.method(Method::GET).with(oauth::oauth_start))
            .resource("/oauth/{service}/callback", |r| r.method(Method::GET).with(oauth::oauth_callback))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)
    } else {
        server.bind("127.0.0.1:8080").unwrap()
    };

    server.run();
}