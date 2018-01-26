use std::net::SocketAddr;

use bytes::Bytes;

use crypto::identity::PublicKey;
use crypto::hash::HashResult;
use crypto::uuid::Uuid;

use futures::sync::mpsc;

use channeler::types::ChannelerNeighborInfo;
use funder::messages::{RequestSendFunds};

use indexer_client::messages::RequestFriendsRoutes;
use proto::indexer::NeighborsRoute;
use proto::funder::InvoiceId;
use proto::networker::{NeighborMoveToken, NeighborRequestType};


/// Indicate the direction of the move token message.
#[derive(Clone, Copy, Debug)]
pub enum MoveTokenDirection {
    Incoming = 0,
    Outgoing = 1,
}

pub enum NeighborStatus {
    Enable = 1,
    Disable = 0,
}

pub struct PendingNeighborRequest {
    pub request_id: Uuid,
    pub route: NeighborsRoute,
    pub request_type: NeighborRequestType,
    pub request_content_hash: HashResult,
    pub max_response_length: u32,
    pub processing_fee_proposal: u64,
    pub credits_per_byte_proposal: u32,
}

/// The neighbor's information from database.
pub struct NeighborInfo {
    pub neighbor_public_key: PublicKey,
    pub neighbor_socket_addr: Option<SocketAddr>,
    pub wanted_remote_max_debt: u64,
    pub wanted_max_channels: u32,
    pub status: NeighborStatus,
}

// ======== Internal Interfaces ========

struct NeighborTokenChannelLoaded {
    channel_index: u32,
    local_max_debt: u64,
    remote_max_debt: u64,
    balance: i64,
}

pub enum NeighborTokenChannelEventInner {
    Open,
    Close,
    LocalMaxDebtChange(u64),  // Contains new local max debt
    RemoteMaxDebtChange(u64), // Contains new remote max debt
    BalanceChange(i64),       // Contains new balance
    InconsistencyError(i64),  // Contains balance required for reset
}

pub struct NeighborLoaded {
    neighbor_public_key: PublicKey,
    neighbor_socket_addr: Option<SocketAddr>,
    max_channels: u32,
    wanted_remote_max_debt: u64,
    status: NeighborStatus,
    token_channels: Vec<NeighborTokenChannelLoaded>,
    // TODO: Should we use a map instead of a vector for token_channels?
}

pub struct NeighborTokenChannelEvent {
    channel_index: u32,
    event: NeighborTokenChannelEventInner,
}

pub enum NeighborEvent {
    NeighborLoaded(NeighborLoaded),
    TokenChannelEvent(NeighborTokenChannelEvent),
}

pub struct NeighborStateUpdate {
    neighbor_public_key: PublicKey,
    event: NeighborEvent,
}

/// The result of attempting to send a message to a remote Networker.
pub enum SendMessageResult {
    Success(Vec<u8>),
    Failure,
}

/// Destination port for the packet.
/// The destination port is used by the destination Networker to know where to forward the received
/// message.
pub enum DestinationPort {
    Funder,
    IndexerClient,
    AppManager(u32),
}

/// Component -> Networker
pub struct RequestSendMessage {
    route: NeighborsRoute,
    dest_port: DestinationPort,
    request_data: Vec<u8>,
    max_response_len: u32,
    processing_fee_proposal: u64,
    credits_per_byte_proposal: u32,
    response_sender: mpsc::Sender<ResponseSendMessage>,
}

/// Networker -> Component
pub struct ResponseSendMessage {
    result: SendMessageResult,
}

/// Networker -> Component
pub struct MessageReceived {
    route: NeighborsRoute, // sender_public_key is the first public key on the NeighborsRoute
    request_data: Vec<u8>,
    max_response_len: u32,
    processing_fee_proposal: u64,
    credits_per_byte_proposal: u32,
    response_sender: mpsc::Sender<MessageReceivedResponse>,
}


/// Component -> Networker
pub enum MessageReceivedResponse {
    Accept {
        response_data: Vec<u8>,
    },
    Discard,
}


pub enum NetworkerToAppManager {
    MessageReceived(MessageReceived),
    ResponseSendMessage(ResponseSendMessage),
    NeighborStateUpdate(NeighborStateUpdate),
}

pub enum NetworkerToChanneler {
    /// Request send message to remote.
    SendChannelMessage {
        neighbor_public_key: PublicKey,
        content: Bytes,
    },
    /// Request to add a new neighbor.
    AddNeighbor {
        info: ChannelerNeighborInfo,
    },
    /// Request to remove a neighbor.
    RemoveNeighbor {
        neighbor_public_key: PublicKey
    },
}

pub enum NetworkerToDatabase {
    StoreNeighbor(NeighborInfo),
    RemoveNeighbor {
        neighbor_public_key: PublicKey,
    },
    RequestLoadNeighbors,
    StoreInNeighborToken {
        neighbor_public_key: PublicKey,
        token_channel_index: u32,
        move_token_message: NeighborMoveToken,
        remote_max_debt: u64,
        local_max_debt: u64,
        remote_pending_debt: u64,
        local_pending_debt: u64,
        balance: i64,
        local_invoice_id: Option<InvoiceId>,
        remote_invoice_id: Option<InvoiceId>,
        closed_local_requests: Vec<Uuid>,
        opened_remote_requests: Vec<PendingNeighborRequest>,
    },
    StoreOutNeighborToken {
        neighbor_public_key: PublicKey,
        move_token_message: NeighborMoveToken,
        remote_max_debt: u64,
        local_max_debt: u64,
        remote_pending_debt: u64,
        local_pending_debt: u64,
        balance: i64,
        local_invoice_id: Option<InvoiceId>,
        remote_invoice_id: Option<InvoiceId>,
        opened_local_requests: Vec<PendingNeighborRequest>,
        closed_remote_requests: Vec<Uuid>,
    },
    RequestLoadNeighborToken {
        neighbor_public_key: PublicKey,
        token_channel_index: u32,
    },
}

pub enum NetworkerToFunder {
    MessageReceived(MessageReceived),
    RequestSendFunds(RequestSendFunds),
}

pub enum NetworkerToIndexerClient {
    MessageReceived(MessageReceived),
    RequestFriendsRoutes(RequestFriendsRoutes),
}
