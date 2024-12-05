use crate::connection::NeoUrl;
use crate::routing::cluster_routing_table::ClusterRoutingTable;
use crate::routing::load_balancing::LoadBalancingStrategy;

pub struct LeastConnectedStrategy {
    cluster_routing_table: ClusterRoutingTable,
}

impl LeastConnectedStrategy {
    pub fn new(cluster_routing_table: ClusterRoutingTable) -> Self {
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