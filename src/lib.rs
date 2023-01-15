#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use anyhow::Result;
use queue::{Queue, Stats};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::broadcast;

mod queue;
pub use queue::Job;

pub struct Redeez {
    client: redis::Client,
    queues: HashMap<String, Queue>,
    shutdown_signal: broadcast::Sender<()>,
}

impl Redeez {
    #[must_use]
    pub fn new(client: redis::Client) -> Self {
        let (shutdown_signal, _) = broadcast::channel(1);

        Self {
            client,
            shutdown_signal,
            queues: HashMap::new(),
        }
    }

    #[must_use]
    pub fn queue(mut self, name: &str, handler: fn(Job) -> Result<()>) -> Self {
        self.queues.insert(
            name.to_string(),
            Queue::new(self.client.clone(), name, handler),
        );

        self
    }

    pub fn dispatch(&self, name: &str, payload: Value) {
        if let Some(queue) = self.queues.get(name) {
            queue.dispatch(payload);
        }
    }

    pub fn listen(&self) {
        let queues = self.queues.clone().into_values();

        for queue in queues {
            let shutdown_signal = self.shutdown_signal.subscribe();

            tokio::spawn(async move { queue.listen(shutdown_signal) });
        }
    }

    #[must_use]
    pub fn stats(&self) -> HashMap<&str, Stats> {
        let mut stats = HashMap::new();

        for (name, queue) in &self.queues {
            stats.insert(name.as_str(), queue.stats().unwrap());
        }

        stats
    }

    pub fn shutdown(&mut self) {
        self.shutdown_signal.send(()).unwrap();
    }
}
