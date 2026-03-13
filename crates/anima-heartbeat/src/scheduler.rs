use std::sync::Arc;
use std::time::Duration;

use anima_skills::SkillContext;

/// Heartbeat scheduler — drives the cognitive loop autonomously.
pub struct Scheduler {
    _ctx: Arc<SkillContext>,
    interval: Duration,
}

impl Scheduler {
    pub fn new(ctx: Arc<SkillContext>, interval_secs: u64) -> Self {
        Self {
            _ctx: ctx,
            interval: Duration::from_secs(interval_secs),
        }
    }

    /// Run the heartbeat loop. This blocks indefinitely.
    pub async fn run(&self) {
        let mut interval = tokio::time::interval(self.interval);
        loop {
            interval.tick().await;
            tracing::info!("heartbeat tick");
            // TODO: Check schedule and execute appropriate skills
            //   - morning briefing (8:00)
            //   - inbox monitoring (every 30 min)
            //   - reminders (every 30 min)
            //   - evening review (18:00)
            //   - cortex maintenance (3:00)
        }
    }
}
