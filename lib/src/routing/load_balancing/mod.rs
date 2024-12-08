pub(crate) mod round_robin_strategy;

use crate::bolt::Server;

pub trait LoadBalancingStrategy: Sync + Send {
    fn select_reader(&self) -> Option<Server>;
    fn select_writer(&self) -> Option<Server>;
    fn select_router(&self) -> Option<Server>;
}
