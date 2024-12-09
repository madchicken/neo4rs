use crate::bolt::{RouteBuilder, RoutingTable};
use crate::pool::ManagedConnection;
use crate::routing::connection_registry::ConnectionRegistry;
use crate::routing::load_balancing::LoadBalancingStrategy;
use crate::{Config, Error};
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use futures::lock::Mutex;
use log::{debug, info};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct RoutedConnectionManager {
    load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    registry: Arc<ConnectionRegistry>,
    bookmarks: Arc<Mutex<Vec<String>>>,
    backoff: Arc<ExponentialBackoff>,
}

pub const WRITE_OPERATION: &'static str = "WRITE";

impl RoutedConnectionManager {
    pub async fn new(
        config: &Config,
        routing_table: Arc<RoutingTable>,
        load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    ) -> Result<Self, Error> {
        let registry = Arc::new(ConnectionRegistry::new(config, routing_table.clone()).await?);
        let backoff = Arc::new(
            ExponentialBackoffBuilder::new()
                .with_initial_interval(Duration::from_millis(1))
                .with_randomization_factor(0.42)
                .with_multiplier(2.0)
                .with_max_elapsed_time(Some(Duration::from_secs(60)))
                .build(),
        );

        Ok(RoutedConnectionManager {
            load_balancing_strategy,
            registry,
            bookmarks: Arc::new(Mutex::new(vec![])),
            backoff,
        })
    }

    pub async fn refresh_routing_table(&self) -> Result<RoutingTable, Error> {
        if let Some(router) = self.load_balancing_strategy.select_router() {
            if let Some(pool) = self.registry.get_pool(&router) {
                if let Ok(mut connection) = pool.get().await {
                    info!(
                        "Refreshing routing table from router {}",
                        router.addresses.first().unwrap()
                    );
                    let bookmarks = self.bookmarks.lock().await;
                    let bookmarks = bookmarks.iter().map(|b| b.as_str()).collect();
                    let route =
                        RouteBuilder::new(router.addresses.first().unwrap().as_str(), bookmarks)
                            .build();
                    match connection.route(route).await {
                        Ok(rt) => {
                            debug!("Routing table refreshed: {:?}", rt);
                            Ok(rt)
                        }
                        Err(e) => Err(Error::RoutingTableRefreshFailed(format!(
                            "Failed to refresh routing table from router {}: {}",
                            router.addresses.first().unwrap(),
                            e
                        ))),
                    }
                } else {
                    Err(Error::RoutingTableRefreshFailed(format!(
                        "Failed to create connection to router {}",
                        router.addresses.first().unwrap()
                    )))
                }
            } else {
                Err(Error::RoutingTableRefreshFailed(
                    "No connection manager available".to_string(),
                ))
            }
        } else {
            Err(Error::RoutingTableRefreshFailed(
                "No router available".to_string(),
            ))
        }
    }

    pub(crate) async fn get(&self, operation: Option<&str>) -> Result<ManagedConnection, Error> {
        // We probably need to do this in a more efficient way, since this will block the request of a connection
        // while we refresh the routing table. We should probably have a separate thread that refreshes the routing
        self.registry
            .update_if_expired(|| self.refresh_routing_table())
            .await?;

        if let Some(router) = match operation.unwrap_or(WRITE_OPERATION) {
            WRITE_OPERATION => self.load_balancing_strategy.select_writer(),
            _ => self.load_balancing_strategy.select_reader(),
        } {
            if let Some(pool) = self.registry.get_pool(&router) {
                Ok(pool.get().await?)
            } else {
                Err(Error::RoutingTableRefreshFailed(
                    "No connection manager available".to_string(),
                ))
            }
        } else {
            Err(Error::RoutingTableRefreshFailed(
                "No router available".to_string(),
            ))
        }
    }

    pub(crate) fn backoff(&self) -> ExponentialBackoff {
        self.backoff.as_ref().clone()
    }
}
