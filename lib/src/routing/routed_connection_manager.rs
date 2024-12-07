use std::sync::{Arc};
use deadpool::managed::{Manager, Metrics, RecycleResult};
use futures::lock::Mutex;
use crate::bolt::{RouteBuilder};
use crate::{Config, Error, RoutingTable};
use crate::connection::Connection;
use crate::routing::connection_registry::ConnectionRegistry;
use crate::routing::load_balancing::LoadBalancingStrategy;

pub struct RoutedConnectionManager {
    routing_table: Arc<RoutingTable>,
    load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    registry: ConnectionRegistry,
    bookmarks: Mutex<Vec<String>>,
    routing_table_fetch_time: Mutex<u64>,
}

impl RoutedConnectionManager {
    pub fn new(
        config: &Config,
        routing_table: Arc<RoutingTable>,
        load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    ) -> Self {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let registry = ConnectionRegistry::new(config, routing_table.clone());
        RoutedConnectionManager {
            routing_table,
            load_balancing_strategy,
            registry,
            bookmarks: Mutex::new(vec![]),
            routing_table_fetch_time: Mutex::new(now),
        }
    }

    pub async fn refresh_routing_table(&mut self) -> Result<RoutingTable, Error> {
        if let Some(router) = self.load_balancing_strategy.select_router() {
            if let Some(connection_manager) = self.registry.connections.get(&router) {
                if let Ok(mut connection) = connection_manager.create().await {
                    let bookmarks = self.bookmarks.lock().await;
                    let bookmarks = bookmarks.iter().map(|b| b.as_str()).collect();
                    let route = RouteBuilder::new(router.addresses.first().unwrap().as_str(), bookmarks).build();
                    match connection.route(route).await {
                        Ok(rt) => {
                            Ok(rt)
                        }
                        Err(e) => {
                            Err(Error::RoutingTableRefreshFailed(format!("Failed to refresh routing table from router {}: {}", router.addresses.first().unwrap(), e)))
                        }
                    }
                } else {
                    Err(Error::RoutingTableRefreshFailed(format!("Failed to create connection to router {}", router.addresses.first().unwrap())))
                }
            } else {
                Err(Error::RoutingTableRefreshFailed("No connection manager available".to_string()))
            }
        } else {
            Err(Error::RoutingTableRefreshFailed("No router available".to_string()))
        }
    }
}

impl Manager for RoutedConnectionManager {
    type Type = Connection;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let router = self.load_balancing_strategy.select_reader().ok_or(Error::ServerUnavailableError("Unable to find an available reader server".to_string()))?;
        let connection_manager = self.registry.connections.get(&router).unwrap();
        connection_manager.create().await
    }

    async fn recycle(&self, obj: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
        Ok(obj.reset().await?)
    }
}