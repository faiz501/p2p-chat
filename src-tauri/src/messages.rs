use std::str::FromStr;

use anyhow::{bail, ensure, Context, Result};
use bytes::Bytes;
use futures_lite::{Stream, StreamExt};
use iroh_docs::rpc::client::docs::Doc;
use iroh_docs::rpc::client::docs::{Entry, LiveEvent, ShareMode};
use iroh_docs::{store::Query, AuthorId, DocTicket};

use quic_rpc::transport::flume::FlumeConnector;
// use iroh::ticket::DocTicket;
use serde::{Deserialize, Serialize};

use super::iroh::Iroh;

/// Todo in a list of todos.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Msg {
    /// String id
    pub id: String,
    /// Description of the todo
    /// Limited to 2000 characters
    pub label: String,
    /// Record creation timestamp. Counted as micros since the Unix epoch.
    pub created: u64,
    /// Whether or not the todo has been completed. Done todos will show up in the todo list until
    /// they are archived.
    /// Indicates whether or not the todo is tombstoned
    pub is_delete: bool,
}

impl Msg {
    fn from_bytes(bytes: Bytes) -> anyhow::Result<Self> {
        let msg = serde_json::from_slice(&bytes).context("invalid json")?;
        Ok(msg)
    }

    fn as_bytes(&self) -> anyhow::Result<Bytes> {
        let buf = serde_json::to_vec(self)?;
        ensure!(buf.len() < MAX_TODO_SIZE, "msg too large");
        Ok(buf.into())
    }

    fn missing_todo(id: String) -> Self {
        Self {
            label: String::from("Missing Content"),
            created: 0,
            is_delete: false,
            id,
        }
    }
}

const MAX_TODO_SIZE: usize = 2 * 1024;
const MAX_LABEL_LEN: usize = 2 * 1000;

/// List of todos, including completed todos that have not been archived
#[derive(Debug)]
pub struct Msgs {
    iroh: Iroh,
    doc: Doc<FlumeConnector<iroh_docs::rpc::proto::Response, iroh_docs::rpc::proto::Request>>,

    ticket: DocTicket,
    author: AuthorId,
}

impl Msgs {
    pub async fn new(ticket: Option<String>, iroh: Iroh) -> anyhow::Result<Self> {
        let author = iroh.docs.authors().create().await?;

        let doc = match ticket {
            None => iroh.docs.create().await?,
            Some(ticket) => {
                let ticket = DocTicket::from_str(&ticket)?;
                iroh.docs.import(ticket).await?
            }
        };

        let ticket = doc.share(ShareMode::Write, Default::default()).await?;

        Ok(Msgs {
            iroh,
            author,
            doc,
            ticket,
        })
    }

    pub fn ticket(&self) -> String {
        self.ticket.to_string()
    }

    pub async fn doc_subscribe(&self) -> Result<impl Stream<Item = Result<LiveEvent>>> {
        self.doc.subscribe().await
    }

    pub async fn add(&mut self, id: String, label: String) -> anyhow::Result<()> {
        if label.len() > MAX_LABEL_LEN {
            bail!("label is too long, max size is {MAX_LABEL_LEN} characters");
        }
        let created = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("time drift")
            .as_secs();
        let msg = Msg {
            label,
            created,
            is_delete: false,
            id: id.clone(),
        };
        self.insert_bytes(id.as_bytes(), msg.as_bytes()?).await
    }

    pub async fn delete(&mut self, id: String) -> anyhow::Result<()> {
        let mut msg = self.get_todo(id.clone()).await?;
        msg.is_delete = true;
        self.update_todo(id.as_bytes(), msg).await
    }

    pub async fn update(&mut self, id: String, label: String) -> anyhow::Result<()> {
        if label.len() >= MAX_LABEL_LEN {
            bail!("label is too long, must be {MAX_LABEL_LEN} or shorter");
        }
        let mut msg = self.get_todo(id.clone()).await?;
        msg.label = label;
        self.update_todo(id.as_bytes(), msg).await
    }

    pub async fn get_msgs(&self) -> anyhow::Result<Vec<Msg>> {
        let mut entries = self.doc.get_many(Query::single_latest_per_key()).await?;

        let mut msgs = Vec::new();
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let msg = self.todo_from_entry(&entry).await?;
            if !msg.is_delete {
                msgs.push(msg);
            }
        }
        msgs.sort_by_key(|t| t.created);
        Ok(msgs)
    }

    async fn insert_bytes(&self, key: impl AsRef<[u8]>, content: Bytes) -> anyhow::Result<()> {
        self.doc
            .set_bytes(self.author, key.as_ref().to_vec(), content)
            .await?;
        Ok(())
    }

    async fn update_todo(&mut self, key: impl AsRef<[u8]>, todo: Msg) -> anyhow::Result<()> {
        let content = todo.as_bytes()?;
        self.insert_bytes(key, content).await
    }

    async fn get_todo(&self, id: String) -> anyhow::Result<Msg> {
        let entry = self
            .doc
            .get_many(Query::single_latest_per_key().key_exact(id))
            .await?
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("no todo found"))??;

        self.todo_from_entry(&entry).await
    }

    async fn todo_from_entry(&self, entry: &Entry) -> anyhow::Result<Msg> {
        let id = String::from_utf8(entry.key().to_owned()).context("invalid key")?;
        match self.iroh.blobs.read_to_bytes(entry.content_hash()).await {
            Ok(b) => Msg::from_bytes(b),
            Err(_) => Ok(Msg::missing_todo(id)),
        }
    }
}
