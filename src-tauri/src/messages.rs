use crate::iroh::Iroh;
use anyhow::{anyhow, Context, Result};
use iroh::NodeId;
use std::str::FromStr;

/// Chat API using Iroh for P2P messaging.
pub struct Chat {
    pub iroh: Iroh,
    /// The remote node id this chat is connected to.
    pub nodeid: Option<NodeId>,
}

impl Chat {
    /// Create a new Chat instance by initializing an Iroh node.
    /// If an ID is provided, join that chat room; otherwise, create a new room.
    pub async fn new(id: Option<String>) -> Result<Self> {
        let iroh = Iroh::new().await?;
        let nodeid = if let Some(id_str) = id {
            let remote_node_id =
                NodeId::from_str(&id_str).context("Invalid NodeId string provided")?;
            // Join an existing chat by connecting to the remote node
            iroh.connect(remote_node_id)
                .await
                .context("Failed to connect to remote peer")?;
            Some(remote_node_id)
        } else {
            // Create a new chat; use our own local node id as the chat's identifier
            Some(iroh.local_node_id())
        };

        Ok(Chat { iroh, nodeid })
    }

    /// Send a message over the Iroh connection.
    pub async fn send(&self, msg: &str) -> Result<()> {
        if let Some(id) = self.nodeid {
            self.iroh.send_msg(&id, msg).await
        } else {
            eprintln!("No node id present, sending failed");
            Err(anyhow!("No node id present, sending failed"))
        }
    }

    /// Receive a message from the Iroh connection.
    pub async fn receive(&self) -> Result<String> {
        let (_peer, msg) = self
            .iroh
            .accept_msg()
            .await
            .context("Error while receiving msg")?;
        Ok(msg)
    }
}
