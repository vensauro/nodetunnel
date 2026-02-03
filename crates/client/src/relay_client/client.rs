use crate::relay_client::events::RelayEvent;
use std::cmp::PartialEq;
use std::time::Duration;
use nt_proto::packet::Packet;
use crate::relay_client::error::RelayClientError;
use crate::transport::client::{ClientEvent, ClientTransport};
use crate::transport::common::{Channel};

#[derive(Debug, PartialEq)]
enum ClientState {
    Connecting,
    Connected,
    Authenticated,
}

pub struct RelayClient {
    transport: Option<ClientTransport>,
    client_state: ClientState,
    last_update: Duration,
}

impl RelayClient {
    pub fn new() -> Self {
        Self {
            transport: None,
            client_state: ClientState::Connecting,
            last_update: Duration::from_secs(0),
        }
    }

    pub fn connect(&mut self, transport: ClientTransport) {
        self.client_state = ClientState::Connecting;
        self.transport = Some(transport);
    }

    pub fn update(&mut self, delta: Duration) -> Result<Vec<RelayEvent>, RelayClientError> {
        let transport = self.transport.as_mut().ok_or(
            RelayClientError::TransportNotInitialized
        )?;

        self.last_update += delta;
        if self.last_update >= Duration::from_secs(5) {
            transport.send_keepalive().expect("TODO: panic message");
            self.last_update = Duration::ZERO;
        }

        let events = transport.recv_packets();

        let mut relay_events = vec![];

        if let Some(event) = self.update_state() {
            relay_events.push(event);
        }

        for event in events {
            if let ClientEvent::PacketReceived { data, channel } = event {
                let packet_events = self.handle_packet(data, channel)?;
                relay_events.extend(packet_events);
            }
        }

        Ok(relay_events)
    }

    fn update_state(&mut self) -> Option<RelayEvent> {
        if self.client_state == ClientState::Connecting && self.is_connected() {
            self.client_state = ClientState::Connected;
            return Some(RelayEvent::ConnectedToServer);
        }

        None
    }

    fn handle_packet(&mut self, data: Vec<u8>, channel: Channel) -> Result<Vec<RelayEvent>, RelayClientError> {
        let mut events = vec![];

        if let Ok(packet_type) = Packet::from_bytes(&data) {
            match packet_type {
                Packet::ClientAuthenticated => {
                    self.client_state = ClientState::Authenticated;
                    events.push(RelayEvent::Authenticated);
                }
                Packet::ConnectedToRoom { room_id, peer_id } =>
                    events.push(RelayEvent::RoomJoined { room_id, peer_id }),
                Packet::GetRooms { rooms } =>
                    events.push(RelayEvent::RoomsReceived { rooms }),
                Packet::PeerJoinAttempt { target_id, metadata } =>
                    events.push(RelayEvent::PeerJoinAttempt { client_id: target_id, metadata } ),
                Packet::PeerJoinedRoom { peer_id } =>
                    events.push(RelayEvent::PeerJoinedRoom { peer_id }),
                Packet::PeerLeftRoom { peer_id } =>
                    events.push(RelayEvent::PeerLeftRoom { peer_id }),
                Packet::GameData { from_peer, data } => {
                    events.push(RelayEvent::GameDataReceived { data, from_peer, channel });
                }
                Packet::ForceDisconnect =>
                    events.push(RelayEvent::ForceDisconnect),
                Packet::Error { error_code, error_message } =>
                    events.push(RelayEvent::Error { error_code, error_message }),
                _ => {
                    return Err(RelayClientError::InvalidPacketType);
                }
            }
        } else {
            return Err(RelayClientError::PacketParsingError);
        }

        Ok(events)
    }

    pub fn req_auth(&mut self, app_id: String) -> Result<(), RelayClientError> {
        self.send_packet(
            Packet::Authenticate {
                app_id,
                version: nt_proto::version::PROTOCOL_VERSION.to_string()
            },
            Channel::Reliable
        )?;

        Ok(())
    }

    pub fn req_create_room(&mut self, is_public: bool, metadata: String) -> Result<(), RelayClientError> {
        self.send_packet(
            Packet::CreateRoom {
                is_public,
                metadata,
            },
            Channel::Reliable
        )?;

        Ok(())
    }

    pub fn req_rooms(&mut self) -> Result<(), RelayClientError> {
        self.send_packet(
            Packet::ReqRooms,
            Channel::Reliable,
        )
    }

    pub fn req_join_room(&mut self, room_id: String, metadata: String) -> Result<(), RelayClientError> {
        self.send_packet(
            Packet::ReqJoin { room_id, metadata },
            Channel::Reliable
        )?;

        Ok(())
    }

    pub fn req_update_room(&mut self, room_id: &str, metadata: &str) -> Result<(), RelayClientError> {
        self.send_packet(
            Packet::UpdateRoom { room_id: room_id.to_string(), metadata: metadata.to_string() },
            Channel::Reliable
        )?;

        Ok(())
    }

    pub fn send_join_response(&mut self, room_id: String, target_id: u64, allowed: bool) -> Result<(), RelayClientError> {
        self.send_packet(
            Packet::JoinRes {
                allowed,
                room_id,
                target_id
            },
            Channel::Reliable
        )?;

        Ok(())
    }

    pub fn send_game_data(&mut self, peer_id: i32, data: Vec<u8>, channel: Channel) -> Result<(), RelayClientError> {
        self.send_packet(
            Packet::GameData { from_peer: peer_id, data },
            channel
        )?;

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.transport.as_ref().map_or(false, |transport| transport.is_connected())
    }

    fn send_packet(&mut self, packet_type: Packet, channel: Channel) -> Result<(), RelayClientError> {
        let transport = self.transport.as_mut().ok_or(
            RelayClientError::TransportNotInitialized
        )?;

        transport.send(
            packet_type.to_bytes(),
            channel,
        ).expect("TODO: panic message");

        Ok(())
    }
}
