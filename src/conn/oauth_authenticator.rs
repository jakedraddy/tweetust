use hyper;
use hyper::client::Response;
use hyper::method::Method;
use oauthcli::{self, SignatureMethod};
use url::Url;
use super::{Authenticator, is_multipart, Parameter, send_request};

/// OAuth 1.0 wrapper
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthAuthenticator {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_token: String,
    pub access_token_secret: String
}

impl OAuthAuthenticator {
    pub fn new(consumer_key: &str, consumer_secret: &str,
        access_token: &str, access_token_secret: &str) -> OAuthAuthenticator
    {
        OAuthAuthenticator {
            consumer_key: consumer_key.to_string(),
            consumer_secret: consumer_secret.to_string(),
            access_token: access_token.to_string(),
            access_token_secret: access_token_secret.to_string()
        }
    }
}

impl Authenticator for OAuthAuthenticator {
    fn send_request(&self, method: Method, url: &str, params: &[Parameter]) -> hyper::Result<Response> {
        match Url::parse(url) {
            Ok(ref u) => {
                let multipart = is_multipart(params);
                let mut auth_params = Vec::<(String, String)>::new();
                if !multipart {
                    auth_params.extend(params.iter().map(|x| match x {
                        &Parameter::Value(key, ref val) => (key.to_string(), val.clone()),
                        _ => unreachable!()
                    }));
                }

                let authorization = oauthcli::authorization_header(
                    &method.to_string()[..],
                    u.clone(),
                    None,
                    &self.consumer_key[..],
                    &self.consumer_secret[..],
                    Some(&self.access_token[..]),
                    Some(&self.access_token_secret[..]),
                    SignatureMethod::HmacSha1,
                    &oauthcli::timestamp()[..],
                    &oauthcli::nonce()[..],
                    None,
                    None,
                    auth_params.into_iter()
                );
                send_request(method, u.clone(), params, authorization)
            },
            Err(e) => Err(hyper::Error::Uri(e))
        }
    }
}
