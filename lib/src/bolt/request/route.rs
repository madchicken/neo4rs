use std::fmt::{format, Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::bolt::{ExpectedResponse, Hello, Summary};
use crate::connection::NeoUrl;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Route<'a> {
    routing: Routing<'a>,
    bookmarks: Vec<&'a str>,
    db: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<Extra<'a>,>
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Routing<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    policy: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<&'a str>,
    address: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Extra<'a> {
    db: &'a str,
    imp_user: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Response {
    pub(crate) rt: RoutingTable,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RoutingTable {
    pub(crate) ttl: u64,
    pub(crate) db: Option<String>,
    pub(crate) servers: Vec<Server>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Server {
    pub(crate) addresses: Vec<String>,
    pub(crate) role: String,
}

impl<'a> ExpectedResponse for Route<'a> {
    type Response = Summary<Response>;
}

pub struct RouteBuilder<'a> {
    routing: Routing<'a>,
    bookmarks: Vec<&'a str>,
    db: Option<&'a str>,
    extra: Option<Extra<'a>>,
}

impl<'a> RouteBuilder<'a> {
    pub fn new(address: &'a str, bookmarks: Vec<&'a str>) -> Self {
        let routing = Routing {
            policy: None,
            region: None,
            address,
        };
        Self {
            routing,
            bookmarks,
            db: None,
            extra: None,
        }
    }

    pub fn db(&mut self, db: &'a str) -> &mut Self {
        self.db = Some(db);
        self
    }

    pub fn extra(&mut self, extra: Extra<'a>) -> &mut Self {
        self.extra = Some(extra);
        self
    }

    pub fn build(self) -> Route<'a> {
        Route {
            routing: self.routing,
            bookmarks: self.bookmarks,
            db: self.db,
            extra: self.extra,
        }
    }
}

impl Display for RoutingTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RoutingTable {{ ttl: {}, db: {}, servers: {} }}", self.ttl, self.db.clone().unwrap_or("null".into()), self.servers.iter().map(|s| s.addresses.join(", ")).collect::<Vec<String>>().join(", ")
        )
    }
}
