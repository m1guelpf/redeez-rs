use anyhow::Result;
use redis::Commands;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Job {
    id: String,
    pub payload: Value,
}

impl Job {
    pub(crate) fn with_data(payload: Value) -> Self {
        Self {
            payload,
            id: Uuid::new_v4().to_string(),
        }
    }

    fn from_string(payload: &str) -> Result<Self> {
        Ok(serde_json::from_str(payload)?)
    }
}

impl Display for Job {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}

#[derive(Clone, Debug)]
pub struct Queue {
    name: String,
    client: redis::Client,
    handler: fn(Job) -> Result<()>,
}

impl Queue {
    pub(crate) fn new(client: redis::Client, name: String, handler: fn(Job) -> Result<()>) -> Self {
        Self {
            name,
            client,
            handler,
        }
    }

    pub(crate) fn dispatch(&self, payload: Value) {
        let mut con = self.client.get_connection().unwrap();

        let _: () = con
            .lpush(&self.name, Job::with_data(payload).to_string())
            .unwrap();
    }

    pub(crate) fn listen(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let mut con = self.client.get_connection().unwrap();

        let failed_queue = format!("{}.failed", &self.name);
        let pending_queue = format!("{}.pending", &self.name);
        let completed_queue = format!("{}.completed", &self.name);

        while shutdown.try_recv().is_err() {
            let Ok(payload) = con.brpoplpush::<_, String>(&self.name, &pending_queue, 5) else {
                continue
            };

            match (self.handler)(Job::from_string(&payload)?) {
                Ok(()) => con.lpush(&completed_queue, payload)?,
                Err(_) => con.lpush(&failed_queue, payload)?,
            }

            con.rpop::<_, ()>(&pending_queue, None).unwrap();
        }

        Ok(())
    }
}
