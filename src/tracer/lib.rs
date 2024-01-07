use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

mod state;
pub mod ui_tracer;
use state::{CollectedEvent, TracerLevel};

#[derive(Debug, Clone)]
pub struct EventCollector {
    level: Level,
    events: Arc<Mutex<Vec<CollectedEvent>>>,
}

impl EventCollector {
    pub fn level(level: Level) -> Self {
        Self {
            level,
            events: Arc::new(Mutex::new(vec![])),
        }
    }

    fn events(&self) -> Vec<CollectedEvent> {
        self.events.lock().unwrap().clone()
    }

    fn clear(&self) {
        let mut events = self.events.lock().unwrap();
        *events = vec![];
    }

    fn collect(&self, event: CollectedEvent) {
        if event.level <= TracerLevel::from(self.level)
            && event.target.starts_with(env!("CARGO_CRATE_NAME"))
        {
            self.events.lock().unwrap().push(event);
        }
    }
}

impl<S> Layer<S> for EventCollector
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, _event: &Event<'_>, _ctx: Context<'_, S>) {
        self.collect(CollectedEvent::new(_event));
    }
}
