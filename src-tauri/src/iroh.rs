use anyhow::{Context, Result};
use iroh::{endpoint::Connection, Endpoint, NodeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Iroh {
    pub endpoint: Endpoint,
    pub connections: Arc<Mutex<HashMap<NodeId, Connection>>>,
}

impl Iroh {
    pub async fn new() -> Result<Self> {
        let endpoint = Endpoint::builder()
            .alpns(vec![b"hello-world".to_vec()])
            .bind()
            .await
            .context("Failed to bind endpoint")?;

        Ok(Iroh {
            endpoint,
            connections: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Returns the local NodeId for this endpoint.
    pub fn local_node_id(&self) -> NodeId {
        // Assumes the underlying endpoint provides a method to retrieve the local node id.
        self.endpoint.node_id()
    }

    pub async fn connect(&self, node_id: NodeId) -> Result<()> {
        let connection = self
            .endpoint
            .connect(node_id, b"hello-world")
            .await
            .context("Failed to connect to peer")?;

        self.connections.lock().await.insert(node_id, connection);

        Ok(())
    }

    pub async fn send_msg(&self, node_id: &NodeId, msg: &str) -> Result<()> {
        let connections = self.connections.lock().await;
        let connection = connections.get(node_id).context("Connection not found")?;

        let mut send_stream = connection
            .open_uni()
            .await
            .context("Failed to open unidirectional stream")?;

        send_stream
            .write_all(msg.as_bytes())
            .await
            .context("Failed to send message")?;

        send_stream.finish().context("Failed to finish stream")?;

        Ok(())
    }

    pub async fn accept_msg(&self) -> Result<(NodeId, String)> {
        let connection = self
            .endpoint
            .accept()
            .await
            .context("No incoming connection")?
            .await
            .context("Failed to accept connection")?;

        let peer_id = connection
            .remote_node_id()
            .context("Could not find peer node id")?;
        self.connections
            .lock()
            .await
            .insert(peer_id, connection.clone());

        let mut recv_stream = connection
            .accept_uni()
            .await
            .context("Failed to accept unidirectional stream")?;

        let mut buf = Vec::new();
        recv_stream
            .read(&mut buf)
            .await
            .context("Failed to read message")?;
        let msg = String::from_utf8(buf).context("Invalid UTF-8 message")?;

        Ok((peer_id, msg))
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.endpoint.close().await;
        Ok(())
    }
}
