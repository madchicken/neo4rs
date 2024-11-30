use neo4rs_macros::BoltStruct;
use crate::{BoltList, BoltMap, BoltNull, BoltString, BoltType};

#[derive(Debug, Clone, BoltStruct, PartialEq)]
#[signature(0x66)]
pub struct Route {
    routing: BoltMap,
    bookmarks: BoltList,
    extra: BoltMap,
}

impl Route {
    pub fn new(routing: BoltMap, bookmarks: Vec<&str>, db: Option<&str>) -> Self {
        let mut extra: BoltMap = Default::default();
        extra.value.insert("db".into(), db.map(|v| BoltType::String(BoltString::new(v))).unwrap_or(BoltType::Null(BoltNull)));
        extra.value.insert("imp_user".into(), BoltType::Null(BoltNull));
        Route {
            routing,
            bookmarks: BoltList::from(bookmarks.into_iter().map(|b| BoltType::String(BoltString::new(b))).collect::<Vec<BoltType>>()),
            extra,
        }
    }
}