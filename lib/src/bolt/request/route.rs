use crate::bolt::{ExpectedResponse, Hello, Summary};
use crate::connection::NeoUrl;
use serde::{Deserialize, Serialize};
use std::fmt::{format, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Route<'a> {
    routing: Routing<'a>,
    bookmarks: Vec<&'a str>,
    db: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<Extra<'a>>,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Hash)]
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
            "RoutingTable {{ ttl: {}, db: {}, servers: {} }}",
            self.ttl,
            self.db.clone().unwrap_or("null".into()),
            self.servers
                .iter()
                .map(|s| s.addresses.join(", "))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::bolt::request::route::Response;
    use crate::bolt::MessageResponse;
    use crate::packstream::bolt;

    #[test]
    fn parse() {
        let data = bolt()
            .tiny_map(1)
            .tiny_string("rt")
            .tiny_map(3)
            .tiny_string("ttl")
            .int64(1000)
            .tiny_string("db")
            .tiny_string("neo4j")
            .tiny_string("servers")
            .tiny_list(1)
            .tiny_map(2)
            .tiny_string("addresses")
            .tiny_list(1)
            .tiny_string("localhost:7687")
            .tiny_string("role")
            .tiny_string("ROUTE")
            .build();

        let response = Response::parse(data).unwrap();

        assert_eq!(response.rt.ttl, 1000);
        assert_eq!(response.rt.db.unwrap(), "neo4j");
        assert_eq!(response.rt.servers.len(), 1);
    }
}
