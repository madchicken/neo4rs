use async_trait::async_trait;
use crate::Error;
use crate::pool::ManagedConnection;

#[async_trait]
pub trait ConnectionProvider: Clone {
    async fn get(&self) -> Result<ManagedConnection, Error>;
}