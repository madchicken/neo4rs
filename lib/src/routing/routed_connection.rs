use crate::connection::{Connection, ConnectionInfo};
use crate::messages::{BoltRequest, BoltResponse, RouteBuilder};
use crate::routing::cluster_routing_table::ClusterRoutingTable;
use crate::{BoltMap, Error};
use async_trait::async_trait;
use log::{debug, error, info};

pub struct RoutedConnection {
    connection: Box<dyn Connection>,
    routing_table: ClusterRoutingTable,
}

impl RoutedConnection {
    pub(crate) async fn new(mut connection: Box<dyn Connection>, info: &ConnectionInfo) -> crate::errors::Result<Self> {
        let mut builder = RouteBuilder::new(&info.routing, vec![]);
        if let Some(database) = info.db.as_ref() {
            builder.with_db(database.clone());
        }
        let req = builder.build(connection.version());
        info!("requesting routing table... {:?}", req);
        match connection.send_recv(req).await? {
            BoltResponse::Success(msg) => {
                let routing_table = ClusterRoutingTable::from(msg.get::<BoltMap>("rt").unwrap());
                debug!("Routing table: {}", routing_table);
                Ok(Self {
                    connection,
                    routing_table,
                })
            },
            BoltResponse::Failure(msg) => {
                error!("Failed to get routing table: {:?}", msg);
                Err(Error::AuthenticationError(msg.get("message").unwrap()))
            }
            msg => Err(msg.into_error("ROUTE")),
        }
    }

    pub(crate) fn connection(&self) -> &dyn Connection {
        self.connection.as_ref()
    }

    pub(crate) fn connection_mut(&mut self) -> &mut dyn Connection {
        self.connection.as_mut()
    }

    pub(crate) fn routing_table(&self) -> &ClusterRoutingTable {
        &self.routing_table
    }

    pub(crate) fn routing_table_mut(&mut self) -> &mut ClusterRoutingTable {
        &mut self.routing_table
    }
}

#[async_trait]
impl Connection for RoutedConnection {
    async fn reset(&mut self) -> crate::Result<()> {
        self.connection.reset().await
    }

    async fn send_recv(&mut self, request: BoltRequest) -> crate::Result<BoltResponse> {
        self.connection.send_recv(request).await
    }

    async fn send(&mut self, message: BoltRequest) -> crate::Result<()> {
        self.connection.send(message).await
    }

    async fn recv(&mut self) -> crate::Result<BoltResponse> {
        self.connection.recv().await
    }

    async fn hello(&mut self, req: BoltRequest) -> crate::Result<()> {
        self.connection.hello(req).await
    }

    fn version(&self) -> crate::Version {
        self.connection.version()
    }
}