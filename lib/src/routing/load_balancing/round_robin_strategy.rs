use std::sync::Mutex;
use crate::bolt::Server;
use crate::RoutingTable;
use crate::routing::load_balancing::LoadBalancingStrategy;

pub struct RoundRobinStrategy {
    readers: Vec<Server>,
    writers: Vec<Server>,
    reader_index: Mutex<usize>,
    writer_index: Mutex<usize>,
}

impl RoundRobinStrategy {
    pub fn new(cluster_routing_table: RoutingTable) -> Self {
        let readers: Vec<Server> = cluster_routing_table.servers.iter().filter(|s| s.role == "READ").cloned().collect();
        let writers: Vec<Server> = cluster_routing_table.servers.iter().filter(|s| s.role == "WRITE").cloned().collect();
        let reader_index = Mutex::new(readers.len());
        let writer_index = Mutex::new(writers.len());
        RoundRobinStrategy {
            readers,
            writers,
            reader_index,
            writer_index,
        }
    }

    fn select(servers: &[Server], index_mutex: &Mutex<usize>) -> Option<Server> {
        if servers.is_empty() {
            return None;
        }

        let mut index = index_mutex.lock().unwrap();
        if index.checked_sub(1).is_none() {
            *index = servers.len();
        }
        *index -= 1;
        let server = servers[*index].clone();
        Some(server)
    }
}

impl LoadBalancingStrategy for RoundRobinStrategy {
    fn select_reader(&self) -> Option<Server> {
        Self::select(&self.readers, &self.reader_index)
    }

    fn select_writer(&self) -> Option<Server> {
        Self::select(&self.writers, &self.reader_index)
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