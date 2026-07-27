use color_eyre::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct EventHandler {
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    #[must_use]
    pub fn new(tick_rate: std::time::Duration) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_rate);

            loop {
                let tick_delay = tick_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    _ = tick_delay => {
                        if sender.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                    Some(Ok(evt)) = crossterm_event => {
                        match evt {
                            CrosstermEvent::Key(key) => {
                                if sender.send(Event::Key(key)).is_err() {
                                    break;
                                }
                            },
                            CrosstermEvent::Mouse(mouse) => {
                                if sender.send(Event::Mouse(mouse)).is_err() {
                                    break;
                                }
                            },
                            CrosstermEvent::Resize(x, y) if sender.send(Event::Resize(x, y)).is_err() => {
                                break;
                            },
                            _ => {}
                        }
                    }
                }
            }
        });

        Self { receiver }
    }

    /// # Errors
    /// Returns an error if the event stream is closed.
    pub async fn next(&mut self) -> Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_else(|| color_eyre::eyre::eyre!("Event stream closed"))
    }
}
