use async_trait::async_trait;
use crate::Error;
use crate::pool::ManagedConnection;

#[async_trait]
pub trait ConnectionProvider {
    async fn acquire(&self) -> Result<ManagedConnection, Error>;
}