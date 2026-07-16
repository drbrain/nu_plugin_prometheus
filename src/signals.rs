use std::time::Duration;

use nu_protocol::{LabeledError, Signals, Span};
use tokio::{
    select,
    sync::oneshot::{Receiver, channel},
    task::JoinHandle,
};

const CHECK_INTERVAL: Duration = Duration::from_millis(10);

pub async fn run_with_signal<'a, F, T>(
    signals: &Signals,
    call_span: Span,
    future: F,
) -> Result<T, LabeledError>
where
    F: Future<Output = T> + 'a,
{
    let (interrupted, task) = signal_wait_channel(signals);

    select! {
        biased;

        _ = interrupted => {
            signals.check(&call_span)?;

            Err(LabeledError::new("Interrupted"))
        }

        result = future => {
            task.abort();

            Ok(result)
        }
    }
}

fn signal_wait_channel(signals: &Signals) -> (Receiver<()>, JoinHandle<()>) {
    let (sender, receiver) = channel();
    let signals = signals.clone();

    let task = tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(CHECK_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            interval.tick().await;

            if signals.interrupted() {
                let _ = sender.send(());
                return;
            }
        }
    });

    (receiver, task)
}
