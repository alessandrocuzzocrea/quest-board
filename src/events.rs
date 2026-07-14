use axum::extract::State;
use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::error::AppError;
use crate::AppState;

/// An event emitted when any resource changes.
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct SseEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub board_id: String,
    pub card_id: Option<String>,
    pub list_id: Option<String>,
    pub comment_id: Option<String>,
    pub user_id: String,
    pub timestamp: String,
}

/// Size of the broadcast channel — keeps this many recent events for late-joining clients.
const CHANNEL_CAPACITY: usize = 256;

/// Create a new broadcast channel for SSE events.
pub fn channel() -> (broadcast::Sender<SseEvent>, broadcast::Receiver<SseEvent>) {
    broadcast::channel(CHANNEL_CAPACITY)
}

/// Send an event through the broadcast channel (best-effort, drops if no receivers).
pub fn emit(tx: &broadcast::Sender<SseEvent>, event: SseEvent) {
    let _ = tx.send(event);
}

/// GET /api/v1/events — SSE endpoint that streams real-time events.
pub async fn event_stream(
    State(state): State<Arc<AppState>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| {
        match result {
            Ok(event) => {
                let json = serde_json::to_string(&event).unwrap_or_default();
                let sse = Event::default()
                    .event("change")
                    .data(json);
                Some(Ok(sse))
            }
            Err(_) => None, // lagged — skip
        }
    });

    Ok(Sse::new(stream))
}

/// Convenience: build a card/list/board event and emit.
pub fn emit_simple(
    tx: &tokio::sync::broadcast::Sender<SseEvent>,
    event_type: &str,
    board_id: &str,
    card_id: Option<&str>,
    list_id: Option<&str>,
    user_id: &str,
) {
    emit(tx, SseEvent {
        event_type: event_type.into(),
        board_id: board_id.into(),
        card_id: card_id.map(String::from),
        list_id: list_id.map(String::from),
        comment_id: None,
        user_id: user_id.into(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    });
}
