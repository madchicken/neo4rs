use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use crate::bolt::Server;
use crate::RoutingTable;
use crate::pool::ConnectionManager;
use crate::routing::load_balancing::LoadBalancingStrategy;

pub struct RoutedConnectionManager {
    routing_table: Arc<Mutex<RoutingTable>>,
    load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    registry: DashMap<Server, ConnectionManager>,
}

impl RoutedConnectionManager {
    pub fn new(
        routing_table: Arc<Mutex<RoutingTable>>,
        load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    ) -> Self {
        RoutedConnectionManager {
            routing_table,
            load_balancing_strategy,
            registry: DashMap::default(),
        }
    }
}