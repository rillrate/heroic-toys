mod checker;
mod heartbeat;
mod interval;

use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context, InterruptedBy, StartedBy, System};
use rillrate::prime::*;
use std::sync::Arc;

pub struct Watcher {
    url: Arc<String>,
    latency: Pulse,
    tail: LiveTail,
    interval: Slider,
}

impl Actor for Watcher {
    type GroupBy = ();
}

impl Watcher {
    pub fn new(url: String) -> Self {
        let latency = Pulse::new(
            "lab.monitor.latency.pulse",
            Default::default(),
            PulseOpts::default()
                .retain(30u32)
                .min(0)
                .max(1_000)
                .higher(true)
                .suffix("ms"),
        );
        let tail = LiveTail::new(
            "lab.monitor.latency.events",
            Default::default(),
            LiveTailOpts::default(),
        );
        let interval = Slider::new(
            "lab.monitor.settings.interval",
            SliderOpts::default()
                .label("Slide Me!")
                .min(1)
                .max(60)
                .step(1),
        );
        Self {
            url: Arc::new(url),
            latency,
            tail,
            interval,
        }
    }
}

#[async_trait]
impl StartedBy<System> for Watcher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.spawn_heartbeat(ctx);
        self.set_interval_callback(ctx);
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<System> for Watcher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
