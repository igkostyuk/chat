use std::{
    collections::HashMap,
    ops::Deref,
    str::FromStr,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use async_trait::async_trait;
use axum::{
    extract::{
        ws::{self, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    response::Response,
};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use shared::domain::{
    event::{ClientEvent, JoinRequest, JoinResponse, ServerEvent, UserJoinResponse},
    Message, MessageContent, RoomId, UserId,
};
use tokio::sync::{broadcast, mpsc};

use crate::service::{self, ChatService};

#[derive(Clone)]
pub struct ChatState<A, C> {
    rooms: Arc<Mutex<HashMap<RoomId, RoomState>>>,
    auth_service: A,
    chat_service: C,
}

struct RoomState {
    tx: broadcast::Sender<ws::Message>,
}

impl<A, C> ChatState<A, C> {
    pub fn new(auth_service: A, chat_service: C) -> Self {
        Self {
            rooms: Arc::new(Mutex::new(HashMap::default())),
            auth_service,
            chat_service,
        }
    }
}

impl<A, C> ChatState<A, C> {
    pub fn get_or_create_room_chanel(&self, id: &RoomId) -> broadcast::Sender<ws::Message> {
        let mut rooms = self.rooms.lock().unwrap();
        let room = rooms.entry(*id).or_insert(RoomState {
            tx: broadcast::channel(1000).0,
        });
        room.tx.clone()
    }
}

pub async fn websocket_handler<A, C>(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ChatState<Arc<A>, Arc<C>>>>,
    Path(room): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, service::Error>
where
    A: service::AuthService + Send + Sync + 'static,
    C: service::ChatService + Send + Sync + 'static,
{
    let token = params
        .get("token")
        .ok_or(service::Error::NotFound("room not found".to_string()))?;

    let room_id = RoomId::from_str(&room)
        .map_err(|_| service::Error::NotFound("room not found".to_string()))?;

    let claims = state.auth_service.validate_token(token).await?;

    let user_id = claims.user_id();

    // let membership = state
    //     .chat_service
    //     .get_membership(&room_id, &user_id)
    //     .await?
    //     .ok_or(service::Error::NotFound("room not found".to_string()))?;

    let code = uuid::Uuid::new_v4().to_string();

    let membership = Membership {
        user_id,
        room_id,
        code,
    };

    Ok(ws.on_upgrade(|socket| websocket(socket, membership, state)))
}

struct Membership {
    pub user_id: UserId,
    pub room_id: RoomId,
    pub code: String,
}

async fn websocket<A, C>(
    stream: WebSocket,
    membership: Membership,
    state: Arc<ChatState<Arc<A>, Arc<C>>>,
) where
    A: service::AuthService + Send + Sync + 'static,
    C: service::ChatService + Send + Sync + 'static,
{
    let (mut sender, mut receiver) = stream.split();

    let room_tx = state.get_or_create_room_chanel(&membership.room_id);

    let mut subscription = room_tx.subscribe();
    let (user_tx, mut rx) = mpsc::channel(100);

    let Membership {
        user_id,
        room_id,
        code,
    } = membership;

    let event_handler = SocketHandler {
        user_id,
        room_id,
        membership_code: code,
        auth_service: state.auth_service.clone(),
        chat_service: state.chat_service.clone(),
        room_tx,
        user_tx: user_tx.clone(),
    };

    let mut subscribe = tokio::spawn(async move {
        while let Ok(msg) = subscription.recv().await {
            if user_tx.send(msg).await.is_err() {
                break;
            }
        }
        tracing::debug!("Close socket from chat send task");
    });

    let mut send = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
        tracing::debug!("Close socket from chat send task");
    });

    let mut recv = tokio::spawn(async move {
        while let Some(Ok(ws::Message::Text(text))) = receiver.next().await {
            if let Ok(event) = serde_json::from_str::<ClientEvent>(&text) {
                if let Err(e) = event_handler.handle(event).await {
                    tracing::error!("Failed handel event: {}", e)
                };
            }
        }
        tracing::debug!("Close socket from recv task");
    });

    tokio::select! {
        _ = (&mut send) => {
            recv.abort();
            subscribe.abort();
        }
        _ = (&mut recv) => {
            send.abort();
            subscribe.abort();
        }
        _ = (&mut subscribe) => {
            send.abort();
            recv.abort();
        }
    }
}

#[derive(Clone)]
pub struct SocketHandler<A, C> {
    pub user_id: UserId,
    pub room_id: RoomId,
    pub membership_code: String,
    pub auth_service: Arc<A>,
    pub chat_service: Arc<C>,
    pub user_tx: mpsc::Sender<ws::Message>,
    pub room_tx: broadcast::Sender<ws::Message>,
}

#[async_trait]
pub trait EventHandler<Ev: ?Sized> {
    async fn handle(&self, event: Ev) -> Result<(), anyhow::Error>;
}

#[async_trait]
impl<A, C> EventHandler<JoinRequest> for SocketHandler<A, C>
where
    C: ChatService + Send + Sync,
    A: Send + Sync,
{
    async fn handle(&self, _ev: JoinRequest) -> Result<(), anyhow::Error> {
        let user_event = ServerEvent::Join(JoinResponse {
            user_id: self.user_id,
        });

        self.user_tx
            .send(WsMessage::try_from(user_event)?.0)
            .await?;

        let room_event = ServerEvent::UserJoin(UserJoinResponse {
            user_id: self.user_id,
        });

        self.room_tx.send(WsMessage::try_from(room_event)?.0)?;

        Ok(())
    }
}

#[async_trait]
impl<A, C> EventHandler<MessageContent> for SocketHandler<A, C>
where
    C: ChatService + Send + Sync,
    A: Send + Sync,
{
    async fn handle(&self, ev: MessageContent) -> Result<(), anyhow::Error> {
        let room_event = ServerEvent::ReceivedMessage(Message {
            id: uuid::Uuid::new_v4().into(),
            user_id: self.user_id,
            room_id: self.room_id,
            content: ev,
            created_at: Utc::now(),
        });

        self.room_tx.send(WsMessage::try_from(room_event)?.0)?;

        Ok(())
    }
}

#[async_trait]
impl<A, C> EventHandler<ClientEvent> for SocketHandler<A, C>
where
    C: ChatService + Send + Sync,
    A: Send + Sync,
{
    async fn handle(&self, ev: ClientEvent) -> Result<(), anyhow::Error> {
        match ev {
            ClientEvent::Join(ev) => self.handle(ev).await,
            ClientEvent::SendMessage(ev) => self.handle(ev).await,
        }
    }
}

struct WsMessage(ws::Message);

impl Deref for WsMessage {
    type Target = ws::Message;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<ServerEvent> for WsMessage {
    type Error = anyhow::Error;

    fn try_from(value: ServerEvent) -> Result<Self, Self::Error> {
        Ok(Self(serde_json::to_string(&value).map(ws::Message::Text)?))
    }
}
