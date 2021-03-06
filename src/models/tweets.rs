use std::cmp;
use std::collections::BTreeMap;
use super::entities::{Entities, ExtendedEntities};
use super::places::Place;
use super::users::User;

#[derive(Clone, Debug, RustcDecodable)]
pub struct Tweet {
    pub contributors: Option<Vec<Contributor>>,
    pub coordinates: Option<Coordinates>,
    pub created_at: String,
    pub current_user_retweet: Option<CurrentUserRetweet>,
    pub entities: Option<Entities>,
    pub extended_entities: Option<ExtendedEntities>,
    pub favorite_count: Option<u32>,
    pub favorited: Option<bool>,
    pub filter_level: Option<String>,
    pub id: i64,
    pub in_reply_to_screen_name: Option<String>,
    pub in_reply_to_status_id: Option<i64>,
    pub in_reply_to_user_id: Option<i64>,
    pub lang: Option<String>,
    pub place: Option<Place>,
    pub possibly_sensitive: Option<bool>,
    //pub scopes: Option<BTreeMap<String, json::Json>>,
    pub retweet_count: u32,
    pub retweeted: Option<bool>,
    pub retweeted_status: Option<Box<Tweet>>,
    pub source: String,
    pub text: String,
    pub user: Option<User>,
    pub withheld_copyright: Option<bool>,
    pub withheld_in_countries: Option<Vec<String>>,
    pub withheld_scope: Option<String>
}

impl cmp::Eq for Tweet { }
impl cmp::PartialEq for Tweet {
    fn eq(&self, other: &Tweet) -> bool { self.id == other.id }
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Contributor {
    pub id: i64,
    pub screen_name: String
}

impl cmp::Eq for Contributor { }
impl cmp::PartialEq for Contributor {
    fn eq(&self, other: &Contributor) -> bool { self.id == other.id }
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Coordinates {
    pub coordinates: Vec<f64>,
    pub type_: String
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, RustcDecodable)]
pub struct CurrentUserRetweet {
    pub id: i64
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct LookupMap {
    pub id: BTreeMap<String, Option<Tweet>>
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct OEmbed {
    pub cache_age: String,
    pub url: String,
    pub provider_url: String,
    pub provider_name: String,
    pub author_name: String,
    pub version: String,
    pub author_url: String,
    pub type_: String,
    pub html: String,
    pub height: Option<i32>,
    pub width: Option<i32>
}
