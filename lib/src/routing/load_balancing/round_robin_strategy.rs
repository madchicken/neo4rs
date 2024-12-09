use crate::bolt::RoutingTable;
use crate::bolt::Server;
use crate::routing::load_balancing::LoadBalancingStrategy;
use std::sync::atomic::AtomicUsize;

pub struct RoundRobinStrategy {
    readers: Vec<Server>,
    writers: Vec<Server>,
    routers: Vec<Server>,
    reader_index: AtomicUsize,
    writer_index: AtomicUsize,
    router_index: AtomicUsize,
}

impl RoundRobinStrategy {
    pub(crate) fn new(cluster_routing_table: RoutingTable) -> Self {
        let readers: Vec<Server> = cluster_routing_table
            .servers
            .iter()
            .filter(|s| s.role == "READ")
            .cloned()
            .collect();
        let writers: Vec<Server> = cluster_routing_table
            .servers
            .iter()
            .filter(|s| s.role == "WRITE")
            .cloned()
            .collect();
        let routers: Vec<Server> = cluster_routing_table
            .servers
            .iter()
            .filter(|s| s.role == "ROUTE")
            .cloned()
            .collect();
        let reader_index = AtomicUsize::new(readers.len());
        let writer_index = AtomicUsize::new(writers.len());
        let router_index = AtomicUsize::new(routers.len());
        RoundRobinStrategy {
            readers,
            writers,
            routers,
            reader_index,
            writer_index,
            router_index,
        }
    }

    fn select(servers: &[Server], index: &AtomicUsize) -> Option<Server> {
        if servers.is_empty() {
            return None;
        }

        index
            .compare_exchange(
                0,
                servers.len(),
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            )
            .ok();
        let i = index.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        let server = servers[i - 1].clone();
        Some(server)
    }
}

impl LoadBalancingStrategy for RoundRobinStrategy {
    fn select_reader(&self) -> Option<Server> {
        Self::select(&self.readers, &self.reader_index)
    }

    fn select_writer(&self) -> Option<Server> {
        Self::select(&self.writers, &self.writer_index)
    }

    fn select_router(&self) -> Option<Server> {
        Self::select(&self.routers, &self.router_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_next_server() {
        let readers = vec![
            Server {
                addresses: vec!["localhost:7687".to_string()],
                role: "READ".to_string(),
            },
            Server {
                addresses: vec!["localhost:7688".to_string()],
                role: "READ".to_string(),
            },
        ];
        let writers = vec![];
        let cluster_routing_table = RoutingTable {
            ttl: 0,
            db: None,
            servers: readers.clone().into_iter().chain(writers.clone()).collect(),
        };
        let strategy = RoundRobinStrategy::new(cluster_routing_table);
        let reader = strategy.select_reader().unwrap();
        assert_eq!(reader, readers[1]);
        let reader = strategy.select_reader().unwrap();
        assert_eq!(reader, readers[0]);
        let reader = strategy.select_reader().unwrap();
        assert_eq!(reader, readers[1]);
        let writer = strategy.select_writer();
        assert_eq!(writer, None);
    }
}
