use std::sync::{Arc};
use std::time::Duration;
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use futures::lock::Mutex;
use crate::bolt::{RouteBuilder};
use crate::{Config, Error, RoutingTable};
use crate::pool::ManagedConnection;
use crate::routing::connection_registry::ConnectionRegistry;
use crate::routing::load_balancing::LoadBalancingStrategy;

#[derive(Clone)]
pub struct RoutedConnectionManager {
    load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    registry: ConnectionRegistry,
    bookmarks: Arc<Mutex<Vec<String>>>,
    backoff: ExponentialBackoff,
}

impl RoutedConnectionManager {
    pub async fn new(
        config: &Config,
        routing_table: Arc<RoutingTable>,
        load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    ) -> Result<Self, Error> {
        let registry = ConnectionRegistry::new(config, routing_table.clone()).await?;
        let backoff = ExponentialBackoffBuilder::new()
            .with_initial_interval(Duration::from_millis(1))
            .with_randomization_factor(0.42)
            .with_multiplier(2.0)
            .with_max_elapsed_time(Some(Duration::from_secs(60)))
            .build();

        Ok(RoutedConnectionManager {
            load_balancing_strategy,
            registry,
            bookmarks: Arc::new(Mutex::new(vec![])),
            backoff,
        })
    }

    pub async fn refresh_routing_table(&self) -> Result<RoutingTable, Error> {
        if let Some(router) = self.load_balancing_strategy.select_router() {
            if let Some(pool) = self.registry.connections.get(&router) {
                if let Ok(mut connection) = pool.get().await {
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

    pub(crate) async fn get(&self, operation: Option<&str>) -> Result<ManagedConnection, Error> {
        if self.registry.is_expired() {
            let _rt = self.refresh_routing_table().await?;
            // TODO: update registry here
        }

        if let Some(router) = match operation.unwrap_or("WRITE") {
            "WRITE" => self.load_balancing_strategy.select_writer(),
            _ => self.load_balancing_strategy.select_reader(),
        } {
            if let Some(pool) = self.registry.connections.get(&router) {
                pool.value().get().await.map_err(Error::from)
            } else {
                Err(Error::RoutingTableRefreshFailed("No connection manager available".to_string()))
            }
        } else {
            Err(Error::RoutingTableRefreshFailed("No router available".to_string()))
        }
    }

    pub(crate) fn backoff(&self) -> ExponentialBackoff {
        self.backoff.clone()
    }
}
