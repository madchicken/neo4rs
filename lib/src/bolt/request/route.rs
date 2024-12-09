use crate::bolt::{ExpectedResponse, Hello, Summary};
use crate::connection::NeoUrl;
use serde::ser::SerializeStructVariant;
use serde::{Deserialize, Serialize};
use std::fmt::{format, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route<'a> {
    routing: Routing<'a>,
    bookmarks: Vec<&'a str>,
    db: Option<String>,
    extra: Option<Extra<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Routing<'a> {
    address: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    policy: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<&'a str>,
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
    pub(crate) role: String, // TODO: use an enum here
}

impl<'a> ExpectedResponse for Route<'a> {
    type Response = Summary<Response>;
}

pub struct RouteBuilder<'a> {
    routing: Routing<'a>,
    bookmarks: Vec<&'a str>,
    db: Option<String>,
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

    pub fn with_db(self, db: String) -> Self {
        Self {
            db: Some(db),
            ..self
        }
    }

    pub fn with_extra(self, extra: Extra<'a>) -> Self {
        Self {
            extra: Some(extra),
            ..self
        }
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

impl<'a> Serialize for Route<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut structure = serializer.serialize_struct_variant("Request", 0x66, "ROUTE", 3)?;
        structure.serialize_field("routing", &self.routing)?;
        structure.serialize_field("bookmarks", &self.bookmarks)?;
        structure.serialize_field("db", &self.db)?;
        structure.end()
    }
}

#[cfg(test)]
mod tests {
    use crate::bolt::request::route::Response;
    use crate::bolt::{Message, MessageResponse, Route, RouteBuilder};
    use crate::packstream::bolt;

    #[test]
    fn serialize() {
        let route = RouteBuilder::new("localhost:7687", vec!["bookmark"])
            .with_db("neo4j".to_string())
            .build();
        let bytes = route.to_bytes().unwrap();

        let expected = bolt()
            .structure(3, 0x66)
            .tiny_map(1)
            .tiny_string("address")
            .tiny_string("localhost:7687")
            .tiny_list(1)
            .tiny_string("bookmark")
            .tiny_string("neo4j")
            .build();

        assert_eq!(bytes, expected);
    }

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
