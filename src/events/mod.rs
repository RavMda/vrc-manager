use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use vrchatapi::models::User;

type EventSender = mpsc::UnboundedSender<AppEvent>;
type EventReceiver = mpsc::UnboundedReceiver<AppEvent>;

#[derive(Clone)]
pub enum AppEvent {
    OnPlayerJoinedRaw(String),
    OnPlayerLeftRaw(String),
    OnAvatarChangedRaw(String),
    OnPlayerJoined(String, User),
    OnPlayerLeft(String, User),
    OnAvatarChanged(String, User),
    OnAutoBanned(String, String), // user_id, avatar_file_id
    OnAutoInvited(String),
}

pub static EVENT_BUS: Lazy<Arc<EventBus>> = Lazy::new(|| Arc::new(EventBus::new()));

pub struct EventBus {
    senders: Mutex<Vec<EventSender>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            senders: Mutex::new(Vec::new()),
        }
    }

    pub async fn subscribe(&self) -> EventReceiver {
        let (tx, rx) = mpsc::unbounded_channel();
        self.senders.lock().await.push(tx);
        rx
    }

    pub async fn publish(&self, event: AppEvent) {
        let mut senders = self.senders.lock().await;
        senders.retain(|sender| sender.send(event.clone()).is_ok());
    }
}

#[macro_export]
macro_rules! listen {
    ($($variant:pat => $handler:expr),+ $(,)?) => {{
        let bus = $crate::events::EVENT_BUS.clone();
        tokio::spawn(async move {
            let mut rx = bus.subscribe().await;
            while let Some(event) = rx.recv().await {
                match event {
                    $($variant => $handler,)+
                    _ => (),
                }
            }
        });
    }};
}
