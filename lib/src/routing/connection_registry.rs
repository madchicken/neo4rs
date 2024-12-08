use crate::bolt::{RoutingTable, Server};
use crate::pool::{create_pool, ConnectionPool};
use crate::{Config, Error};
use dashmap::DashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub(crate) struct ConnectionRegistry {
    config: Config,
    creation_time: u64,
    ttl: u64,
    pub(crate) connections: Arc<RwLock<DashMap<Server, ConnectionPool>>>,
}

impl ConnectionRegistry {
    pub(crate) async fn new(
        config: &Config,
        routing_table: Arc<RoutingTable>,
    ) -> Result<Self, Error> {
        let mut connections = DashMap::new();
        let ttl = routing_table.ttl;
        Self::build_registry(config, routing_table, &mut connections).await?;
        Ok(ConnectionRegistry {
            config: config.clone(),
            creation_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl,
            connections: Arc::new(RwLock::new(connections)),
        })
    }

    async fn build_registry(
        config: &Config,
        routing_table: Arc<RoutingTable>,
        registry: &mut DashMap<Server, ConnectionPool>,
    ) -> Result<(), Error> {
        let servers = routing_table.servers.clone();
        for server in servers.iter() {
            if registry.contains_key(server) {
                continue;
            }
            registry.insert(server.clone(), create_pool(config).await?);
        }
        registry.retain(|k, _| !servers.contains(k));
        Ok(())
    }

    pub(crate) fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.creation_time > self.ttl
    }

    pub(crate) async fn update(&self, routing_table: RoutingTable) -> Result<(), Error> {
        let mut connections = self.connections.write().unwrap();
        Self::build_registry(&self.config, Arc::new(routing_table), &mut connections).await?;
        Ok(())
    }
}
