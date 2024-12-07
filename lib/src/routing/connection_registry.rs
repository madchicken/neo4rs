use std::cell::RefMut;
use std::sync::Arc;
use dashmap::DashMap;
use crate::bolt::{RoutingTable, Server};
use crate::Config;
use crate::pool::ConnectionManager;

pub(crate) struct ConnectionRegistry {
    creation_time: u64,
    ttl: u64,
    config: Arc<Config>,
    pub(crate) connections: DashMap<Server, ConnectionManager>,
}

impl ConnectionRegistry {
    pub(crate) fn new(config: &Config, routing_table: Arc<RoutingTable>) -> Self {
        let mut connections = DashMap::new();
        let ttl = routing_table.ttl;
        Self::build_registry(config, routing_table, &mut connections);
        ConnectionRegistry {
            creation_time: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            ttl,
            config: Arc::new(config.clone()),
            connections,
        }
    }

    fn build_registry(config: &Config, routing_table: Arc<RoutingTable>, registry: &mut DashMap<Server, ConnectionManager>) {
        let servers = routing_table.servers.clone();
        for server in servers.iter() {
            if registry.contains_key(&server) {
                continue;
            }
            let connection_manager = ConnectionManager::new(
                &config.uri,
                &config.user,
                &config.password,
                config.db.clone(),
                &config.tls_config,
            ).unwrap();
            registry.insert(server.clone(), connection_manager);
        }
        registry.retain(|k, _| !servers.contains(k));
    }

    pub(crate) fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        now - self.creation_time > self.ttl
    }
}