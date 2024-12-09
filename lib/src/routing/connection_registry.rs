use crate::bolt::{RoutingTable, Server};
use crate::pool::{create_pool, ConnectionPool};
use crate::{Config, Error};
use dashmap::DashMap;
use futures::lock::Mutex;
use log::{debug, info};
use std::sync::Arc;

pub type Registry = DashMap<Server, ConnectionPool>;

#[derive(Clone)]
pub(crate) struct ConnectionRegistry {
    config: Config,
    creation_time: Arc<Mutex<u64>>,
    ttl: u64,
    pub(crate) connections: Registry, // Arc is needed for Clone
}

impl ConnectionRegistry {
    pub(crate) async fn new(
        config: &Config,
        routing_table: Arc<RoutingTable>,
    ) -> Result<Self, Error> {
        let ttl = routing_table.ttl;
        let connections = Self::build_registry(config, routing_table).await?;
        Ok(ConnectionRegistry {
            config: config.clone(),
            creation_time: Arc::new(Mutex::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )),
            ttl,
            connections,
        })
    }

    async fn build_registry(
        config: &Config,
        routing_table: Arc<RoutingTable>,
    ) -> Result<Registry, Error> {
        let registry = DashMap::new();
        let servers = routing_table.servers.clone();
        for server in servers.iter() {
            registry.insert(server.clone(), create_pool(config).await?);
        }
        Ok(registry)
    }

    pub(crate) async fn update_if_expired<F, R>(&self, f: F) -> Result<(), Error>
    where
        F: FnOnce() -> R,
        R: std::future::Future<Output = Result<RoutingTable, Error>>,
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        info!("Checking if routing table is expired...");
        let mut guard = self.creation_time.lock().await;
        if now - *guard > self.ttl {
            info!("Routing table expired, refreshing...");
            let routing_table = f().await?;
            info!("Routing table refreshed: {:?}", routing_table);
            let registry = &self.connections;
            let servers = routing_table.servers.clone();
            for server in servers.iter() {
                if registry.contains_key(server) {
                    continue;
                }
                registry.insert(server.clone(), create_pool(&self.config).await?);
            }
            registry.retain(|k, _| servers.contains(k));
            info!("Registry updated. New size is {}", registry.len());
            *guard = now;
        }
        Ok(())
    }
    /// Retrieve the pool for a specific server.
    pub fn get_pool(&self, server: &Server) -> Option<ConnectionPool> {
        self.connections.get(server).map(|entry| entry.clone())
    }

    pub fn mark_unavailable(&self, server: &Server) {
        self.connections.remove(server);
    }
}

const _: () = {
    const fn assert_send_sync<T: ?Sized + Send + Sync>() {}
    assert_send_sync::<Registry>();
};
