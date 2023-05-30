#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
//! # Redeez
//! A simplified general-purpose queueing system for Rust apps.
//!
//!
//! # Example
//!
//! ```no_run
//! use redeez::{Job, Redeez};
//! use serde_json::json;
//! use anyhow::Result;
//!
//! #[tokio::main]
//! async fn main() {
//!     let redis = redis::Client::open("redis://127:0.0.1:6379").unwrap();
//!
//!     let mut queues = Redeez::new(redis)
//!         .queue("external_fn", external_fn)
//!         .queue("test", |job| -> Result<()> {
//!             println!("Received job: {:?}", job);
//!
//!             // -- snip --
//!
//!             Ok(())
//!         });
//!
//!     // Start listening for new jobs.
//!     queues.listen();
//!
//!     queues.dispatch("test", json!({ "foo": "bar" }));
//!
//!     // -- snip --
//!
//!    // Shutdown the queues.
//!     queues.shutdown();
//! }
//! ```

use error::Result;
use queue::Queue;
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::broadcast;

mod queue;
mod error;
pub use queue::{Job, Stats};
pub use error::Error;

/// A general-purpose queueing system for Rust apps.
pub struct Redeez {
    client: redis::Client,
    queues: HashMap<String, Queue>,
    shutdown_signal: broadcast::Sender<()>,
}

impl Redeez {
    /// Creates a new Redeez instance.
    #[must_use]
    pub fn new(client: redis::Client) -> Self {
        let (shutdown_signal, _) = broadcast::channel(1);

        Self {
            client,
            shutdown_signal,
            queues: HashMap::new(),
        }
    }

    /// Adds a new queue to the Redeez instance.
    #[must_use]
    pub fn queue(mut self, name: &str, handler: fn(Job) -> Result<()>) -> Self {
        self.queues.insert(
            name.to_string(),
            Queue::new(self.client.clone(), name, handler),
        );

        self
    }

    /// Dispatches a job to the queue with the given name.
    pub fn dispatch(&self, name: &str, payload: Value) {
        if let Some(queue) = self.queues.get(name) {
            queue.dispatch(payload);
        }
    }

    /// Starts listening for new jobs on all queues.
    pub fn listen(&self) {
        let queues = self.queues.clone().into_values();

        for queue in queues {
            let shutdown_signal = self.shutdown_signal.subscribe();

            tokio::spawn(async move { queue.listen(shutdown_signal) });
        }
    }

    /// Returns a map of queue names to their stats.
    ///
    /// # Panics
    ///
    /// This function will panic if any of the queues fail to return their stats.
    #[must_use]
    pub fn stats(&self) -> HashMap<&str, Stats> {
        let mut stats = HashMap::new();

        for (name, queue) in &self.queues {
            stats.insert(name.as_str(), queue.stats().unwrap());
        }

        stats
    }

    /// Shuts down the Redeez instance and stops listening for new jobs.
    ///
    /// # Panics
    ///
    /// This function will panic if the shutdown signal fails to send.
    pub fn shutdown(&mut self) {
        self.shutdown_signal.send(()).unwrap();
    }
}
