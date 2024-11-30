#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoltServerAddress {
    pub(crate) host: String,
    pub(crate) connection_host: String,
    pub(crate) port: u16,
}

impl BoltServerAddress {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        let string   = host.into();
        Self {
            host: string.clone(),
            connection_host: string,
            port,
        }
    }
    
    pub fn with_connection_host(mut self, connection_host: impl Into<String>) -> Self {
        self.connection_host = connection_host.into();
        self
    }
}