use neo4rs_macros::BoltStruct;
use crate::{BoltList, BoltMap, BoltString, BoltType};

#[derive(Debug, Clone, BoltStruct, PartialEq)]
#[signature(0xB3, 0x66)]
pub struct Route {
    routing: BoltMap,
    bookmarks: BoltList,
    db: BoltString,
}

impl Route {
    pub fn new(routing: BoltMap, bookmarks: Vec<&str>, db: Option<&str>) -> Self {
        Route {
            routing,
            bookmarks: BoltList::from(bookmarks.into_iter().map(|b| BoltType::String(BoltString::new(b))).collect::<Vec<BoltType>>()),
            db: BoltString::from(db.unwrap_or("")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;
    use bytes::*;
    use crate::connection::Routing;
    use crate::types::{list, map, string, BoltWireFormat};

    #[test]
    fn should_serialize_route() {
        let opt: Option<BoltMap> = Routing::Yes(vec![("address".into(), "localhost".into())].into_iter().collect()).into();
        let route = Route::new(
            opt.unwrap(),
            vec![],
            None,
        );

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
