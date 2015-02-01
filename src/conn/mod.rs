//! The low-level functions for connecting to Twitter with any authorization.
//! Usually, you will not use this module.

use std::fmt::{self, Writer};
use std::rc::Rc;
use std::string::ToString;
use hyper;
use hyper::{header, mime, HttpError, HttpResult, Get, Delete, Head};
use hyper::client::Response;
use hyper::method::Method;
use hyper::status::StatusClass;
use oauthcli;
use rustc_serialize::{json, Decodable};
use url::{percent_encoding, Url};
use ::{TwitterError, TwitterResult};
use models::*;
use models::error::{Error, ErrorResponse};

pub mod application_only_authenticator;
pub mod oauth_authenticator;

pub enum Parameter<'a> {
    Value(&'a str, String),
    File(&'a str, &'a mut (Reader + 'a))
}

pub trait Authenticator: Clone {
    fn send_request(&self, method: Method, url: &str, params: &[Parameter])
        -> HttpResult<Response>;

    fn request_twitter(&self, method: Method, url: &str, params: &[Parameter])
        -> TwitterResult<()>
    {
        read_to_twitter_result(self.send_request(method, url, params))
    }
}

fn is_multipart(params: &[Parameter]) -> bool {
    params.iter().any(|x| match *x {
        Parameter::Value(..) => false,
        Parameter::File(..) => true
    })
}

fn create_query<'a, I>(mut pairs: I) -> String
    where I: Iterator<Item=(&'a str, &'a str)>
{
    let es = oauthcli::encode_set();
    let mut s = String::new();
    for (key, val) in pairs {
        if s.len() > 0 {
            s.push('&');
        }
        percent_encoding::utf8_percent_encode_to(key, es, &mut s);
        s.push('=');
        percent_encoding::utf8_percent_encode_to(val, es, &mut s);
    }
    s
}

pub fn send_request(method: Method, mut url: Url, params: &[Parameter],
    authorization: String) -> HttpResult<Response>
{
    let has_body = match method {
        Get | Delete | Head => false,
        _ => true
    };

    if !has_body {
        let query = match url.query_pairs() {
            Some(x) => x,
            None => Vec::new()
        };
        url.query = Some(create_query(
            query.iter().map(|x| (x.0.as_slice(), x.1.as_slice())).chain(
                params.iter().map(|x| match x {
                    &Parameter::Value(key, ref val) => (key, val.as_slice()),
                    _ => panic!("the request whose method is GET, DELETE or HEAD has Parameter::File")
                })
            )
        ));
    }

    let mut client = hyper::Client::new();
    let body;
    let mut req = client.request(method, url);

    if has_body {
        if is_multipart(params) {
            unimplemented!();
        } else {
            body = create_query(
                params.iter().map(|x| match x {
                    &Parameter::Value(key, ref val) => (key, val.as_slice()),
                    _ => unreachable!()
                })
            );
            req = req.body(body.as_slice())
                .header(header::ContentType(mime::Mime(
                    mime::TopLevel::Application,
                    mime::SubLevel::WwwFormUrlEncoded,
                    Vec::new()
                )));
        }
    }

    req.header(header::Authorization(authorization)).send()
}

#[derive(Debug, RustcDecodable)]
struct InternalErrorResponse {
    errors: Option<Vec<Error>>,
    error: Option<Vec<Error>>
}

fn read_to_twitter_result(source: HttpResult<Response>) -> TwitterResult<()> {
    match source {
        Ok(mut res) => {
            // Parse headers
            let limit = res.headers.get_raw("X-Rate-Limit-Limit")
                .and_then(|x| x.first())
                .and_then(|x| String::from_utf8_lossy(x.as_slice()).as_slice().parse());
            let remaining = res.headers.get_raw("X-Rate-Limit-Remaining")
                .and_then(|x| x.first())
                .and_then(|x| String::from_utf8_lossy(x.as_slice()).as_slice().parse());
            let reset = res.headers.get_raw("X-Rate-Limit-Reset")
                .and_then(|x| x.first())
                .and_then(|x| String::from_utf8_lossy(x.as_slice()).as_slice().parse());
            let rate_limit = limit.and(remaining).and(reset)
                .map(|_| RateLimitStatus {
                    limit: limit.unwrap(),
                    remaining: remaining.unwrap(),
                    reset: reset.unwrap()
                });

            match res.read_to_string() {
                Ok(body) => match res.status.class() {
                    // 2xx
                    StatusClass::Success => Ok(TwitterResponse {
                        object: (), raw_response: Rc::new(body), rate_limit: rate_limit
                    }),
                    _ => {
                        // Error response
                        let dec: json::DecodeResult<InternalErrorResponse> = json::decode(body.as_slice());
                        let errors = dec.ok().and_then(|x| x.errors.or(x.error));
                        Err(TwitterError::ErrorResponse(ErrorResponse {
                            status: res.status,
                            errors: errors,
                            raw_response: body,
                            rate_limit: rate_limit
                        }))
                    }
                },
                Err(e) => Err(TwitterError::HttpError(HttpError::HttpIoError(e)))
            }
        },
        Err(e) => Err(TwitterError::HttpError(e))
    }
}

pub fn request_twitter(method: Method, url: Url, params: &[Parameter],
    authorization: String) -> TwitterResult<()>
{
    read_to_twitter_result(send_request(method, url, params, authorization))
}

pub trait ToParameter {
    fn to_parameter<'a>(self, key: &'a str) -> Parameter<'a>;
}

impl<T: ToString> ToParameter for T {
    fn to_parameter<'a>(self, key: &'a str) -> Parameter<'a> {
        Parameter::Value(key, self.to_string())
    }
}

impl <T: fmt::Display> ToParameter for Vec<T> {
    fn to_parameter<'a>(self, key: &'a str) -> Parameter<'a> {
        let mut val = String::new();
        for elm in self.into_iter() {
            if val.len() > 0 {
                val.push(',');
            }
            write!(&mut val, "{}", elm);
        }

        Parameter::Value(key, val)
    }
}

/// Parse the JSON string to T with rustc-serialize.
/// As a stopgap measure, this function renames `type` to `type_`.
pub fn parse_json<T: Decodable>(s: &str) -> json::DecodeResult<T> {
    json::decode(s.replace("\"type\":", "\"type_\":").as_slice())
}
