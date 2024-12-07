use std::sync::Arc;
use dashmap::DashMap;
use crate::bolt::{RoutingTable, Server};
use crate::{Config, Error};
use crate::pool::{create_pool, ConnectionPool};

#[derive(Clone)]
pub(crate) struct ConnectionRegistry {
    creation_time: u64,
    ttl: u64,
    pub(crate) connections: Arc<DashMap<Server, ConnectionPool>>,
}

impl ConnectionRegistry {
    pub(crate) async fn new(config: &Config, routing_table: Arc<RoutingTable>) -> Result<Self, Error> {
        let mut connections = DashMap::new();
        let ttl = routing_table.ttl;
        Self::build_registry(config, routing_table, &mut connections).await?;
        Ok(ConnectionRegistry {
            creation_time: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            ttl,
            connections: Arc::new(connections),
        })
    }

    async fn build_registry(config: &Config, routing_table: Arc<RoutingTable>, registry: &mut DashMap<Server, ConnectionPool>) -> Result<(), Error> {
        let servers = routing_table.servers.clone();
        for server in servers.iter() {
            if registry.contains_key(&server) {
                continue;
            }
            registry.insert(server.clone(), create_pool(&config).await?);
        }
        registry.retain(|k, _| !servers.contains(k));
        Ok(())
    }

    pub(crate) fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        now - self.creation_time > self.ttl
    }
}