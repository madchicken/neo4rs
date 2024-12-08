use crate::{BoltList, BoltMap, BoltString, BoltType, Database};
use neo4rs_macros::BoltStruct;

#[derive(Debug, Clone, BoltStruct, PartialEq)]
#[signature(0xB3, 0x66)]
pub struct Route {
    routing: BoltMap,
    bookmarks: BoltList,
    db: BoltString, // TODO: this can also be null. How do we represent a null string?
}

impl Route {
    pub fn new(routing: BoltMap, bookmarks: Vec<&str>, db: Option<Database>) -> Self {
        Route {
            routing,
            bookmarks: BoltList::from(
                bookmarks
                    .into_iter()
                    .map(|b| BoltType::String(BoltString::new(b)))
                    .collect::<Vec<BoltType>>(),
            ),
            db: BoltString::from(db.map(|d| d.to_string()).unwrap_or("".to_string())),
        }
    }
}

/*
impl From<BoltMap> for RoutingTable {
    fn from(rt: BoltMap) -> Self {
        let mut builder = ClusterRoutingTableBuilder::new();
        let ttl = rt.get::<i64>("ttl").unwrap_or(0);
        let db = rt.get::<String>("db").unwrap_or_default();
        builder.with_database(db).with_expiration_time(ttl);
        let servers = rt.get::<Vec<BoltMap>>("servers").unwrap_or_default();
        for server in servers {
            let role = server.get::<String>("role").unwrap_or_default();
            let addresses = server.get::<Vec<String>>("addresses").unwrap_or_default();
            let addresses = addresses
                .iter()
                .map(|address| NeoUrl::parse(address).unwrap())
                .collect::<Vec<_>>();
            match role.as_str() {
                "ROUTE" => {
                    builder.with_routers(addresses);
                }
                "WRITE" => {
                    builder.with_writers(addresses);
                }
                "READ" => {
                    builder.with_readers(addresses);
                }
                _ => {}
            }
        }
        builder.build()
    }
}
 */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::Routing;
    use crate::types::{list, map, string, BoltWireFormat};
    use crate::version::Version;
    use bytes::*;

    #[test]
    fn should_serialize_route() {
        let opt: Option<BoltMap> = Routing::Yes(
            vec![("address".into(), "localhost".into())]
                .into_iter()
                .collect(),
        )
        .into();
        let route = Route::new(opt.unwrap(), vec![], None);

        let bytes: Bytes = route.into_bytes(Version::V4_1).unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB3,
                0x66,
                map::TINY | 1,
                string::TINY | 7,
                b'a',
                b'd',
                b'd',
                b'r',
                b'e',
                b's',
                b's',
                string::TINY | 9,
                b'l',
                b'o',
                b'c',
                b'a',
                b'l',
                b'h',
                b'o',
                b's',
                b't',
                list::TINY | 0,
                string::TINY | 0,
            ])
        );
    }
}
