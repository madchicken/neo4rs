use crate::routing::routing_table::RoutingTable;
use crate::routing::server_address::BoltServerAddress;

pub struct ClusterRoutingTable {
    routers: Vec<BoltServerAddress>,
    readers: Vec<BoltServerAddress>,
    writers: Vec<BoltServerAddress>,
    servers: Vec<BoltServerAddress>,
    database: String,
    expiration_time: i64,
}

impl ClusterRoutingTable {
    pub fn new(
        routers: Vec<BoltServerAddress>,
        readers: Vec<BoltServerAddress>,
        writers: Vec<BoltServerAddress>,
        servers: Vec<BoltServerAddress>,
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
    fn readers(&self) -> Vec<BoltServerAddress> {
        self.readers.clone()
    }

    fn writers(&self) -> Vec<BoltServerAddress> {
        self.writers.clone()
    }

    fn routers(&self) -> Vec<BoltServerAddress> {
        self.routers.clone()
    }

    fn servers(&self) -> Vec<BoltServerAddress> {
        self.servers.clone()
    }

    fn database(&self) -> String {
        self.database.clone()
    }

    fn expiration_time(&self) -> i64 {
        self.expiration_time
    }
}