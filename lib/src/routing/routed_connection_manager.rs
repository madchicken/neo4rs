use crate::pool::ManagedConnection;
use crate::routing::connection_registry::ConnectionRegistry;
use crate::routing::load_balancing::LoadBalancingStrategy;
use crate::{Config, Error, Operation};
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use futures::lock::Mutex;
use log::{debug, error, info};
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use {
    crate::connection::Routing,
    crate::routing::{RouteBuilder, RoutingTable},
};

#[derive(Clone)]
pub struct RoutedConnectionManager {
    load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    registry: Arc<ConnectionRegistry>,
    bookmarks: Arc<Mutex<Vec<String>>>,
    backoff: Arc<ExponentialBackoff>,
    config: Config,
}

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
            config: config.clone(),
        })
    }

    pub async fn refresh_routing_table(&self) -> Result<RoutingTable, Error> {
        while let Some(router) = self
            .load_balancing_strategy
            .select_router(self.registry.servers().as_slice())
        {
            if let Some(pool) = self.registry.get_pool(&router) {
                if let Ok(mut connection) = pool.get().await {
                    info!(
                        "Refreshing routing table from router {}",
                        router.addresses.first().unwrap()
                    );
                    let bookmarks = self.bookmarks.lock().await;
                    let bookmarks = bookmarks.iter().map(|b| b.as_str()).collect();
                    let route = RouteBuilder::new(Routing::Yes(vec![]), bookmarks)
                        .with_db(self.config.db.clone().unwrap_or_default())
                        .build(connection.version());
                    match connection.route(route).await {
                        Ok(rt) => {
                            debug!("Routing table refreshed: {:?}", rt);
                            return Ok(rt);
                        }
                        Err(e) => {
                            self.registry.mark_unavailable(&router);
                            error!(
                                "Failed to refresh routing table from router {}: {}",
                                router.addresses.first().unwrap(),
                                e
                            );
                        }
                    }
                } else {
                    self.registry.mark_unavailable(&router);
                    error!(
                        "Failed to create connection to router `{}`",
                        router.addresses.first().unwrap()
                    );
                }
            } else {
                error!(
                    "No connection manager available for router `{}` in the registry. Maybe it was marked as unavailable",
                    router.addresses.first().unwrap()
                );
            }
        }
        // After trying all routers, we still couldn't refresh the routing table: return an error
        Err(Error::ServerUnavailableError(
            "No router available".to_string(),
        ))
    }

    pub(crate) async fn get(
        &self,
        operation: Option<Operation>,
    ) -> Result<ManagedConnection, Error> {
        // We probably need to do this in a more efficient way, since this will block the request of a connection
        // while we refresh the routing table. We should probably have a separate thread that refreshes the routing
        self.registry
            .update_if_expired(|| self.refresh_routing_table())
            .await?;

        let op = operation.unwrap_or(Operation::Write);
        let available_servers = self.registry.servers();
        while let Some(server) = match op {
            Operation::Write => self
                .load_balancing_strategy
                .select_writer(available_servers.as_slice()),
            _ => self
                .load_balancing_strategy
                .select_reader(available_servers.as_slice()),
        } {
            if let Some(pool) = self.registry.get_pool(&server) {
                match pool.get().await {
                    Ok(connection) => return Ok(connection),
                    Err(e) => {
                        error!(
                            "Failed to get connection from pool for server `{}`: {}",
                            server.addresses.first().unwrap(),
                            e
                        );
                        self.registry.mark_unavailable(&server);
                        continue;
                    }
                }
            } else {
                // We couldn't find a connection manager for the server, it was probably marked unavailable
                error!(
                    "No connection manager available for router `{}` in the registry",
                    server.addresses.first().unwrap()
                );
            }
        }
        Err(Error::RoutingTableRefreshFailed(format!(
            "No server available for {op} operation"
        )))
    }

    pub(crate) fn backoff(&self) -> ExponentialBackoff {
        self.backoff.as_ref().clone()
    }
}