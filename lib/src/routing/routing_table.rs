use crate::connection::NeoUrl;

pub trait RoutingTable {
    fn readers(&self) -> Vec<NeoUrl>;
    fn writers(&self) -> Vec<NeoUrl>;
    fn routers(&self) -> Vec<NeoUrl>;
    fn servers(&self) -> Vec<NeoUrl>;
    fn database(&self) -> String;
    fn expiration_time(&self) -> i64;
}