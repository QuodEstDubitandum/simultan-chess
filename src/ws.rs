use std::sync::Arc;

use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use log::{error, info};
use uuid::Uuid;

use crate::utils::request::MoveRequest;
use crate::{
    game::{chess_piece::Color, GameResult},
    server::Server,
    utils::request::MoveResponse,
};

#[derive(Clone)]
pub struct WebSocketRoom {
    game_id: Uuid,
    connections: Vec<Addr<WebSocketConnection>>,
    server: Arc<Server>,
}

impl Actor for WebSocketRoom {
    type Context = ws::WebsocketContext<Self>;
}

impl WebSocketRoom {
    pub fn new(game_id: Uuid, server: Arc<Server>) -> WebSocketRoom {
        WebSocketRoom {
            game_id,
            connections: Vec::new(),
            server,
        }
    }
    pub fn add_connection(&mut self, addr: Addr<WebSocketConnection>) {
        self.connections.push(addr);
    }
    pub fn remove_connection(&mut self, addr: Addr<WebSocketConnection>) {
        self.connections.retain(|conn| *conn != addr);
    }
    pub fn broadcast_message(&self, text: String) {
        if text == "white" || text == "black" {
            self.connections.iter().for_each(|e| {
                let _ = e.do_send(FinishBroadcastMessage::new(text.clone()));
            });
            return;
        }
        if let Ok(move_request) = serde_json::from_str::<MoveRequest>(&text) {
            let mut games = self.server.games.write().unwrap();
            let game = match games.get_mut(&self.game_id) {
                None => {
                    error!(
                        "Could not find game when broadcasting message with id {}",
                        self.game_id
                    );
                    return;
                }
                Some(game) => game,
            };

            let promotion_piece = match move_request.promotion.chars().next() {
                None => {
                    error!("No promotion piece char specified");
                    return;
                }
                Some(pr) => pr,
            };

            if let Err(e) =
                &game.validate_and_make_move(&move_request.from, &move_request.to, promotion_piece)
            {
                error!("Not a valid move: {}", e);
                return;
            }
            info!("Move {} is valid", &game.previous_move);

            let player_str = match game.next_to_move {
                Color::WHITE => "BLACK",
                Color::BLACK => "WHITE",
            };

            let db_clone = self.server.db.clone();
            let turn_number = game.turn_number;
            let prev_move = game.previous_move.clone();
            let game_id = self.game_id.clone();
            let result = game.game_result.clone();
            actix::spawn(async move {
                let _ = db_clone
                    .insert_move(turn_number, &game_id.to_string(), &prev_move, player_str)
                    .await;
                match result {
                    None => (),
                    Some(GameResult::WhiteWon) => {
                        info!("White won, finishing game automatically...");
                        let _ = db_clone.finish_game("1-0", &game_id.to_string()).await;
                    }
                    Some(GameResult::BlackWon) => {
                        info!("Black won, finishing game automatically...");
                        let _ = db_clone.finish_game("0-1", &game_id.to_string()).await;
                    }
                }
            });

            let move_response = &MoveResponse {
                move_notation: game.previous_move.clone(),
                player: game.next_to_move.opposite_color(),
                from: move_request.from,
                to: move_request.to,
                promotion: move_request.promotion,
                en_passant: game.previous_move_was_enpassant,
                result: match game.game_result {
                    None => None,
                    Some(res) => Some(res.to_str()),
                },
            };

            self.connections.iter().for_each(|e| {
                let _ = e.do_send(BroadcastMessage::new(move_response.clone()));
            });
        }
    }
}

#[derive(Clone)]
pub struct WebSocketConnection {
    game_id: Uuid,
    server: Arc<Server>,
}

impl Actor for WebSocketConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        info!("New websocket connected for game_id {}", &self.game_id);

        let mut rooms = self.server.rooms.write().unwrap();
        let room = match rooms.get_mut(&self.game_id) {
            None => {
                error!(
                    "Could not find game when adding connection with id {}",
                    self.game_id
                );
                return;
            }
            Some(room) => room,
        };

        room.add_connection(addr);
    }
    fn stopped(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        info!("New websocket disconnected for game_id {}", &self.game_id);

        let mut rooms = self.server.rooms.write().unwrap();
        let room = match rooms.get_mut(&self.game_id) {
            None => {
                error!(
                    "Could not find game when removing connection with id {}",
                    self.game_id
                );
                return;
            }
            Some(room) => room,
        };

        room.remove_connection(addr);
    }
}

impl WebSocketConnection {
    pub fn new(game_id: Uuid, server: Arc<Server>) -> WebSocketConnection {
        WebSocketConnection { game_id, server }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let rooms = self.server.rooms.read().unwrap();
                match rooms.get(&self.game_id) {
                    None => (),
                    Some(room) => room.broadcast_message(text.to_string()),
                }
            }
            _ => (),
        }
    }
}

pub struct BroadcastMessage(MoveResponse);
impl Message for BroadcastMessage {
    type Result = ();
}
impl BroadcastMessage {
    pub fn new(move_response: MoveResponse) -> BroadcastMessage {
        BroadcastMessage(move_response)
    }
}
impl Handler<BroadcastMessage> for WebSocketConnection {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string::<MoveResponse>(&msg.0).unwrap());
    }
}

pub struct FinishBroadcastMessage(String);
impl Message for FinishBroadcastMessage {
    type Result = ();
}
impl FinishBroadcastMessage {
    pub fn new(text: String) -> FinishBroadcastMessage {
        FinishBroadcastMessage(text)
    }
}
impl Handler<FinishBroadcastMessage> for WebSocketConnection {
    type Result = ();

    fn handle(&mut self, msg: FinishBroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
