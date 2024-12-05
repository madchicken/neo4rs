use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::connection::NeoUrl;
use crate::pool::ConnectionManager;
use crate::routing::load_balancing::LoadBalancingStrategy;
use crate::routing::routing_table::RoutingTable;

pub struct RoutedConnectionManager {
    routing_table: Arc<Mutex<dyn RoutingTable>>,
    load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    registry: HashMap<NeoUrl, ConnectionManager>,
}

impl RoutedConnectionManager {
    pub fn new(
        routing_table: Arc<Mutex<dyn RoutingTable>>,
        load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    ) -> Self {
        RoutedConnectionManager {
            routing_table,
            load_balancing_strategy,
            registry: HashMap::new(),
        }
    }
}