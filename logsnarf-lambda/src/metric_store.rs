use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use tokio::sync::Notify;
use tokio::time::{self, Duration, Instant};

use tracing::info;

// use crate::Result;
use crate::Metric;

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

const FLUSH_TIMEOUT: Duration = Duration::new(10, 0);

pub type Token = String;

#[derive(Debug)]
pub struct MetricStore {
    // data: RwLock<HashMap<Token, Item>>,
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
    background_task: Notify,
}

#[derive(Debug)]
struct State {
    metrics: HashMap<Token, Entry>,
    flush_timers: BTreeMap<Instant, Token>,
    shutdown: bool,
}

#[derive(Debug, Default)]
struct Entry {
    data: Vec<Metric>,
}

impl MetricStore {
    pub fn new() -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                metrics: HashMap::with_capacity(100),
                flush_timers: BTreeMap::new(),
                shutdown: false,
            }),
            background_task: Notify::new(),
        });

        tokio::spawn(flush_timered_metrics(shared.clone()));

        Self { shared }
    }

    pub fn push(&self, token: Token, metrics: Vec<Metric>) -> Result<(), E> {
        let mut state = self.shared.state.lock().unwrap();

        let notify = state.push(token, metrics);

        drop(state);

        if notify {
            self.shared.background_task.notify_one();
        };

        Ok(())
    }

    pub fn flush_all(&self) -> Result<(), E> {
        let mut state = self.shared.state.lock().unwrap();
        state.flush_all();
        Ok(())
    }
}

impl State {
    fn push(&mut self, token: Token, metrics: Vec<Metric>) -> bool {
        let mut notify = false;

        let entry = self.metrics.entry(token.to_owned()).or_default();

        if entry.data.is_empty() {
            notify = true;
            let flush_at = Instant::now() + FLUSH_TIMEOUT;
            info!("Setting flush timer {} {:?}", token, flush_at);
            self.flush_timers.insert(flush_at, token.clone());
        };

        entry.push(metrics);

        notify
    }

    fn flush_all(&mut self) {
        info!("Flushing all metrics!");
        while let Some((&when, token)) = self.flush_timers.iter().next() {
            if let Some(entry) = self.metrics.get_mut(token) {
                entry.flush();
                self.flush_timers.remove(&when);
            }
        }
    }
}

impl Entry {
    fn push(&mut self, metrics: Vec<Metric>) {
        self.data.extend(metrics.into_iter());
    }

    fn flush(&mut self) {
        info!("Flushing: {:?}", self.data);
        self.data.clear();
    }
}

impl Shared {
    fn flush_timered_metrics(&self) -> Option<Instant> { 
        let mut state = self.state.lock().unwrap();

        if state.shutdown {
            return None;
        }

        // This is needed to make the borrow checker happy. In short, `lock()`
        // returns a `MutexGuard` and not a `&mut State`. The borrow checker is
        // not able to see "through" the mutex guard and determine that it is
        // safe to access both `state.expirations` and `state.entries` mutably,
        // so we get a "real" mutable reference to `State` outside of the loop.
        let state = &mut *state;

        let now = Instant::now();

        while let Some((&when, token)) = state.flush_timers.iter().next() {
            if when > now {
                return Some(when);
            }

            // time to flush
            if let Some(entry) = state.metrics.get_mut(token) {
                entry.flush();
                state.flush_timers.remove(&when);
            }
        }

        None

    }

    fn is_shutdown(&self) -> bool {
        self.state.lock().unwrap().shutdown
    }
}

async fn flush_timered_metrics(shared: Arc<Shared>) {
    while !shared.is_shutdown() {
        if let Some(when) = shared.flush_timered_metrics() {
            tokio::select! {
                _ = time::sleep_until(when) => {}
                _ = shared.background_task.notified() => {}
            }
        } else {
            shared.background_task.notified().await;
        }
    }

    info!("Flush timer background task shut down");
}

