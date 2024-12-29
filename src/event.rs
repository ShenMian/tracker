use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

/// Terminal events.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Event {
    Update,
    Render,
    Key(KeyEvent),
    Mouse(MouseEvent),
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    #[expect(dead_code)]
    sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
    /// Event handler thread.
    #[expect(dead_code)]
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new() -> Self {
        const UPDATE_RATE: f64 = 10.0;
        const RENDER_RATE: f64 = 60.0;

        let update_period = Duration::from_secs_f64(1.0 / UPDATE_RATE);
        let render_period = Duration::from_secs_f64(1.0 / RENDER_RATE);
        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();
        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut update_interval = tokio::time::interval(update_period);
            let mut render_interval = tokio::time::interval(render_period);
            loop {
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  _ = _sender.closed() => {
                    break;
                  }
                  _ = update_interval.tick() => {
                    _sender.send(Event::Update).unwrap();
                  }
                  _ = render_interval.tick() => {
                    _sender.send(Event::Render).unwrap();
                  }
                  Some(Ok(event)) = crossterm_event => {
                    match event {
                      CrosstermEvent::Key(key) => {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                          _sender.send(Event::Key(key)).unwrap();
                        }
                      },
                      CrosstermEvent::Mouse(mouse) => {
                        _sender.send(Event::Mouse(mouse)).unwrap();
                      },
                      _ => {},
                    }
                  }
                };
            }
        });
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub async fn next(&mut self) -> Result<Event> {
        self.receiver.recv().await.ok_or(
          anyhow::anyhow!("the event receiver has been closed and there are no remaining messages in the receiver's buffer")
        )
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
