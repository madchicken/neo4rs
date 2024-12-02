use std::fmt::{Display, Formatter};
use crate::BoltMap;
use crate::connection::NeoUrl;
use crate::routing::routing_table::RoutingTable;

#[derive(Debug, Clone, Default)]
pub struct ClusterRoutingTable {
    routers: Vec<NeoUrl>,
    readers: Vec<NeoUrl>,
    writers: Vec<NeoUrl>,
    servers: Vec<NeoUrl>,
    database: String,
    expiration_time: i64,
}

impl ClusterRoutingTable {
    pub fn new(
        routers: Vec<NeoUrl>,
        readers: Vec<NeoUrl>,
        writers: Vec<NeoUrl>,
        servers: Vec<NeoUrl>,
        database: String,
        expiration_time: i64,
    ) -> Self {
        ClusterRoutingTable {
            routers,
            readers,
            writers,
            servers,
            database,
            expiration_time,
        }
    }
}

impl RoutingTable for ClusterRoutingTable {
    fn readers(&self) -> Vec<NeoUrl> {
        self.readers.clone()
    }

    fn writers(&self) -> Vec<NeoUrl> {
        self.writers.clone()
    }

    fn routers(&self) -> Vec<NeoUrl> {
        self.routers.clone()
    }

    fn servers(&self) -> Vec<NeoUrl> {
        self.servers.clone()
    }

    fn database(&self) -> String {
        self.database.clone()
    }

    fn expiration_time(&self) -> i64 {
        self.expiration_time
    }
}

pub(crate) struct ClusterRoutingTableBuilder {
    routers: Vec<NeoUrl>,
    readers: Vec<NeoUrl>,
    writers: Vec<NeoUrl>,
    servers: Vec<NeoUrl>,
    database: String,
    expiration_time: i64,
}

impl ClusterRoutingTableBuilder {
    pub fn new() -> Self {
        ClusterRoutingTableBuilder {
            routers: Vec::new(),
            readers: Vec::new(),
            writers: Vec::new(),
            servers: Vec::new(),
            database: String::new(),
            expiration_time: 0,
        }
    }

    pub fn with_routers(&mut self, routers: Vec<NeoUrl>) -> &mut Self {
        self.routers = routers;
        self
    }

    pub fn with_readers(&mut self, readers: Vec<NeoUrl>) -> &mut Self {
        self.readers = readers;
        self
    }

    pub fn with_writers(&mut self, writers: Vec<NeoUrl>) -> &mut Self {
        self.writers = writers;
        self
    }

    pub fn with_servers(&mut self, server: NeoUrl) -> &mut Self {
        self.servers.push(server);
        self
    }

    pub fn with_database(&mut self, database: String) -> &mut Self {
        self.database = database;
        self
    }

    pub fn with_expiration_time(&mut self, expiration_time: i64) -> &mut Self {
        self.expiration_time = expiration_time;
        self
    }

    pub fn build(self) -> ClusterRoutingTable {
        ClusterRoutingTable {
            routers: self.routers,
            readers: self.readers,
            writers: self.writers,
            servers: self.servers,
            database: self.database,
            expiration_time: self.expiration_time,
        }
    }
}

impl From<BoltMap> for ClusterRoutingTable {
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

impl Display for ClusterRoutingTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClusterRoutingTable {{ routers: {:?}, readers: {:?}, writers: {:?}, servers: {:?}, database: {}, expiration_time: {} }}",
            self.routers, self.readers, self.writers, self.servers, self.database, self.expiration_time
        )
    }
}