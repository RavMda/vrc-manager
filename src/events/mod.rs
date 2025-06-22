use once_cell::sync::Lazy;
use std::{any::Any, sync::Arc};
use tokio::sync::{Mutex, mpsc};

pub static BUS: Lazy<Arc<EventBus>> = Lazy::new(|| Arc::new(EventBus::new()));

pub struct EventBus {
    senders: Mutex<Vec<mpsc::UnboundedSender<Arc<dyn Any + Send + Sync>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            senders: Mutex::new(Vec::new()),
        }
    }

    pub async fn subscribe(&self) -> mpsc::UnboundedReceiver<Arc<dyn Any + Send + Sync>> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.senders.lock().await.push(tx);
        rx
    }

    pub async fn publish<T: Any + Send + Sync>(&self, message: T) {
        let message = Arc::new(message) as Arc<dyn Any + Send + Sync>;
        let mut senders = self.senders.lock().await;
        senders.retain(|sender| sender.send(message.clone()).is_ok());
    }
}

#[macro_export]
macro_rules! listen {
    ($($variant:pat => $handler:expr),+ $(,)?) => {{
        let bus = $crate::events::BUS.clone();
        tokio::spawn(async move {
            let mut rx = bus.subscribe().await;
            while let Some(event) = rx.recv().await {
                match event.downcast_ref() {
                    $(Some($variant) => $handler,)+
                    _ => (),
                }
            }
        });
    }};
}

pub enum AppEvent {
    OnPlayerJoined(String),
    OnPlayerLeft(String),
    OnAvatarChanged(String),
}
