mod least_connected_strategy;

use crate::connection::NeoUrl;

pub trait LoadBalancingStrategy {
    fn select_reader(&self) -> Option<NeoUrl>;
    fn select_writer(&self) -> Option<NeoUrl>;
}