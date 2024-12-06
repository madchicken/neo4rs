use crate::RoutingTable;
use crate::connection::NeoUrl;
use crate::routing::load_balancing::LoadBalancingStrategy;

pub struct LeastConnectedStrategy {
    cluster_routing_table: RoutingTable,
}

impl LeastConnectedStrategy {
    pub fn new(cluster_routing_table: RoutingTable) -> Self {
        LeastConnectedStrategy {
            cluster_routing_table,
        }
    }
}

impl LoadBalancingStrategy for LeastConnectedStrategy {
    fn select_reader(&self) -> Option<NeoUrl> {
        // some implementation omitted
        None
    }

    fn select_writer(&self) -> Option<NeoUrl> {
        // some implementation omitted
        None
    }
}