extern crate actix_web;
extern crate listenfd;
extern crate reqwest;
extern crate url;
extern crate hyper;
extern crate base64;

use listenfd::ListenFd;
use actix_web::{http::Method, http::header, server, App, HttpRequest, HttpResponse, Path, Query};
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use std::collections::HashMap;

fn index(_req: &HttpRequest) -> &'static str {
    "Hello world!"
}

fn oauth_flow(service: Path<(String)>) -> HttpResponse {
    let fitbit_client_id = "22DFW9";
    let oauth_redirect_uri = "http://localhost:8080/oauth/fitbit/callback";
    let scopes = ["activity", "heartrate", "location", "profile", "sleep", "weight"].join(" ");
    let redirect_uri = format!("https://www.fitbit.com/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&expires_in={}",
        utf8_percent_encode(fitbit_client_id, DEFAULT_ENCODE_SET),
        utf8_percent_encode(oauth_redirect_uri, DEFAULT_ENCODE_SET),
        utf8_percent_encode(&scopes, DEFAULT_ENCODE_SET),
        // 1 hour
        604800);

    HttpResponse::Found()
        .header(header::LOCATION, redirect_uri)
        .finish()
}

fn oauth_callback(service: Path<(String)>, query: Query<HashMap<String, String>>) -> HttpResponse {
    let client = reqwest::Client::new();

    let code = match query.get("code") {
        Some(c) => Ok(c),
        None => Err("No callback code!")
    };

    let response = match code {
        Ok(c) => client.post("https://api.fitbit.com/oauth2/token")
            .basic_auth("22DFW9", Some("NOPE"))
            .form(&[("clientId", "22DFW9"),
                    ("grant_type", "authorization_code"),
                    ("redirect_uri", &(utf8_percent_encode("http://localhost:8080/oauth/fitbit/callback", DEFAULT_ENCODE_SET).to_string())),
                    ("code", c)])
            .send(),
        Err(e) => unimplemented!()
    };

    let text = match response {
        Ok(mut r) => match r.text() {
            Ok(t) => t,
            Err(e) => unimplemented!()
        },
        Err(e) => unimplemented!()
    };

    HttpResponse::Ok()
        .body(text)
}

fn main() {
    let mut listenfd = ListenFd::from_env();
    let mut server = server::new(|| {
        App::new()
            .resource("/", |r| r.f(index))
            .resource("/oauth/{service}/start", |r| r.method(Method::GET).with(oauth_flow))
            .resource("/oauth/{service}/callback", |r| r.method(Method::GET).with(oauth_callback))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)
    } else {
        server.bind("127.0.0.1:8080").unwrap()
    };

    server.run();
}