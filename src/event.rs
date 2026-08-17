use color_eyre::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use notify::{Event as NotifyEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Internal event type wrapping terminal input, system signals, and filesystem notifications.
#[derive(Clone, Debug)]
pub enum Event {
    /// Periodic tick event.
    Tick,
    /// Terminal key press event.
    Key(KeyEvent),
    /// Terminal mouse event.
    Mouse(MouseEvent),
    /// Terminal window resize event.
    Resize(u16, u16),
    /// External filesystem modification event for a note file.
    FileChanged(PathBuf),
    /// Background check found a new version available.
    NewVersion(String),
}

/// Asynchronous event loop listener that polls Crossterm terminal events and file system changes.
#[derive(Debug)]
pub struct EventHandler {
    receiver: mpsc::UnboundedReceiver<Event>,
    _watcher: Option<RecommendedWatcher>,
}

impl EventHandler {
    /// Creates a new `EventHandler` with given tick rate and optional notes directory file watcher.
    #[must_use]
    pub fn new(tick_rate: std::time::Duration, watch_dir: Option<PathBuf>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let sender_clone = sender.clone();

        let mut watcher = None;
        if let Some(dir) = watch_dir
            && let Ok(mut w) =
                notify::recommended_watcher(move |res: notify::Result<NotifyEvent>| {
                    if let Ok(event) = res
                        && (event.kind.is_modify()
                            || event.kind.is_create()
                            || event.kind.is_remove())
                    {
                        for path in event.paths {
                            let _ = sender_clone.send(Event::FileChanged(path));
                        }
                    }
                })
        {
            let _ = w.watch(&dir, RecursiveMode::NonRecursive);
            watcher = Some(w);
        }

        let sender_version = sender.clone();

        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_rate);
            let mut last_key_event: Option<(KeyEvent, std::time::Instant)> = None;
            let debounce_duration = std::time::Duration::from_millis(100);

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
                                // Only process key if it's different from the last key or enough time has passed
                                let now = std::time::Instant::now();
                                let should_process = last_key_event.is_none_or(|(last_key, last_time)| {
                                    last_key != key || now.duration_since(last_time) >= debounce_duration
                                });

                                if should_process {
                                    last_key_event = Some((key, now));
                                    if sender.send(Event::Key(key)).is_err() {
                                        break;
                                    }
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

        // Spawn version check task
        tokio::spawn(async move {
            if let Ok(resp) =
                reqwest::get("https://raw.githubusercontent.com/tareqmy/ferronote/master/.version")
                    .await
                && let Ok(text) = resp.text().await
            {
                let version = text.trim().to_string();
                let current_version = include_str!("../.version").trim().to_string();

                let parse = |v: &str| {
                    v.split('.')
                        .filter_map(|s| s.parse::<u32>().ok())
                        .collect::<Vec<_>>()
                };
                if !version.is_empty() && parse(&version) > parse(&current_version) {
                    let _ = sender_version.send(Event::NewVersion(version));
                }
            }
        });

        Self {
            receiver,
            _watcher: watcher,
        }
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
