use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::RoutingTable;
use crate::connection::NeoUrl;
use crate::pool::ConnectionManager;
use crate::routing::load_balancing::LoadBalancingStrategy;

pub struct RoutedConnectionManager {
    routing_table: Arc<Mutex<RoutingTable>>,
    load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    registry: HashMap<NeoUrl, ConnectionManager>,
}

impl RoutedConnectionManager {
    pub fn new(
        routing_table: Arc<Mutex<RoutingTable>>,
        load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    ) -> Self {
        RoutedConnectionManager {
            routing_table,
            load_balancing_strategy,
            registry: HashMap::new(),
        }
    }
}