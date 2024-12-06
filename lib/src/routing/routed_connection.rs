use crate::RoutingTable;
use crate::connection::Connection;

pub struct RoutedConnection {
    connection: Box<Connection>,
    routing_table: RoutingTable,
}

impl RoutedConnection {
    pub(crate) async fn new(connection: Box<Connection>, rt: &RoutingTable) -> Self {
        Self {
            connection,
            routing_table: rt.clone(),
        }
    }

    pub(crate) fn connection(&self) -> &Connection {
        self.connection.as_ref()
    }

    pub(crate) fn connection_mut(&mut self) -> &mut Connection {
        self.connection.as_mut()
    }

    pub(crate) fn routing_table(&self) -> &RoutingTable {
        &self.routing_table
    }

    pub(crate) fn routing_table_mut(&mut self) -> &mut RoutingTable {
        &mut self.routing_table
    }
}
