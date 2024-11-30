use crate::routing::server_address::BoltServerAddress;

pub trait RoutingTable {
    fn readers(&self) -> Vec<BoltServerAddress>;
    fn writers(&self) -> Vec<BoltServerAddress>;
    fn routers(&self) -> Vec<BoltServerAddress>;
    fn servers(&self) -> Vec<BoltServerAddress>;
    fn database(&self) -> String;
    fn expiration_time(&self) -> i64;
}