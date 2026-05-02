use crossterm::event::{Event as CtEvent, EventStream, KeyEvent};
use futures::StreamExt;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{interval, Duration};

use crate::models::LogEvent;

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Paste(String),
    Resize,  // terminal size change; handled by ratatui automatically
    Tick,
    Log(LogEvent),
}

pub fn spawn_event_task(
    log_rx: broadcast::Receiver<LogEvent>,
    tx: mpsc::Sender<Event>,
) {
    tokio::spawn(async move {
        run_event_loop(log_rx, tx).await;
    });
}

async fn run_event_loop(
    mut log_rx: broadcast::Receiver<LogEvent>,
    tx: mpsc::Sender<Event>,
) {
    let mut stream = EventStream::new();
    let mut tick = interval(Duration::from_millis(250));

    loop {
        tokio::select! {
            maybe_event = stream.next() => {
                match maybe_event {
                    Some(Ok(CtEvent::Key(key))) => {
                        if tx.send(Event::Key(key)).await.is_err() { break; }
                    }
                    Some(Ok(CtEvent::Paste(s))) => {
                        if tx.send(Event::Paste(s)).await.is_err() { break; }
                    }
                    Some(Ok(CtEvent::Resize(_, _))) => {
                        if tx.send(Event::Resize).await.is_err() { break; }
                    }
                    None | Some(Err(_)) => break,
                    _ => {}
                }
            }
            _ = tick.tick() => {
                if tx.send(Event::Tick).await.is_err() { break; }
            }
            result = log_rx.recv() => {
                match result {
                    Ok(ev) => { let _ = tx.send(Event::Log(ev)).await; }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        }
    }
}
