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
pub struct Keys {
    pub failed: String,
    pub pending: String,
    pub recovery: String,
    pub completed: String,
}

impl Keys {
    fn new(name: &str) -> Self {
        Self {
            pending: name.to_string(),
            failed: format!("{}.failed", &name),
            recovery: format!("{}.pending", &name),
            completed: format!("{}.completed", &name),
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Stats {
    pub failed: usize,
    pub pending: usize,
    pub completed: usize,
    pub processing: usize,
}

#[derive(Clone, Debug)]
pub struct Queue {
    pub queues: Keys,
    pub client: redis::Client,
    handler: fn(Job) -> Result<()>,
}

impl Queue {
    pub(crate) fn new(client: redis::Client, name: &str, handler: fn(Job) -> Result<()>) -> Self {
        Self {
            client,
            handler,
            queues: Keys::new(name),
        }
    }

    pub(crate) fn stats(&self) -> Result<Stats> {
        let mut con = self.client.get_connection().unwrap();

        let (pending, failed, completed, processing): (usize, usize, usize, usize) = redis::pipe()
            .llen(&self.queues.pending)
            .llen(&self.queues.failed)
            .llen(&self.queues.completed)
            .llen(&self.queues.recovery)
            .query(&mut con)?;

        Ok(Stats {
            failed,
            pending,
            completed,
            processing,
        })
    }

    pub(crate) fn dispatch(&self, payload: Value) {
        let mut con = self.client.get_connection().unwrap();

        let _: () = con
            .lpush(&self.queues.pending, Job::with_data(payload).to_string())
            .unwrap();
    }

    pub(crate) fn listen(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let mut con = self.client.get_connection().unwrap();

        while shutdown.try_recv().is_err() {
            let Ok(payload) = con.brpoplpush::<_, String>(&self.queues.pending, &self.queues.recovery, 5) else {
                continue
            };

            match (self.handler)(Job::from_string(&payload)?) {
                Ok(()) => con.lpush(&self.queues.completed, &payload)?,
                Err(_) => con.lpush(&self.queues.failed, &payload)?,
            }

            con.lrem::<_, _, ()>(&self.queues.recovery, 1, &payload)?;
        }

        Ok(())
    }
}
