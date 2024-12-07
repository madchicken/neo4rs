mod round_robin_strategy;

use crate::bolt::Server;

pub trait LoadBalancingStrategy {
    fn select_reader(&self) -> Option<Server>;
    fn select_writer(&self) -> Option<Server>;
}