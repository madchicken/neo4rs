use crate::bolt::{RoutingTable, Server};
use crate::pool::{create_pool, ConnectionPool};
use crate::{Config, Error};
use dashmap::DashMap;
use std::sync::Arc;

pub type Registry = DashMap<Server, ConnectionPool>;

#[derive(Clone)]
pub(crate) struct ConnectionRegistry {
    config: Config,
    creation_time: u64,
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
            creation_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
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

    pub(crate) fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.creation_time > self.ttl
    }

    pub(crate) async fn update(&self, routing_table: RoutingTable) -> Result<(), Error> {
        let registry = &self.connections;
        let servers = routing_table.servers.clone();
        for server in servers.iter() {
            if registry.contains_key(server) {
                continue;
            }
            registry.insert(server.clone(), create_pool(&self.config).await?);
        }
        registry.retain(|k, _| !servers.contains(k));
        Ok(())
    }
    /// Retrieve the pool for a specific server.
    pub fn get_pool(&self, server: &Server) -> Option<ConnectionPool> {
        self.connections.get(server).map(|entry| entry.clone())
    }
}

const _: () = {
    const fn assert_send_sync<T: ?Sized + Send + Sync>() {}
    assert_send_sync::<Registry>();
};
