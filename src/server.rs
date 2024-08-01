use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use log::error;
use uuid::Uuid;

use crate::{
    db::DB,
    game::{chess_piece::Color, Game},
    utils::error::INTERNAL_SERVER_ERROR,
    ws::WebSocketRoom,
};

pub struct Server {
    pub games: Arc<RwLock<HashMap<Uuid, Game>>>,
    pub rooms: Arc<RwLock<HashMap<Uuid, WebSocketRoom>>>,
    pub db: Arc<DB>,
}
impl Server {
    pub async fn new() -> Server {
        Server {
            games: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
            db: Arc::new(DB::new().await),
        }
    }
    pub async fn add_game(
        self: &Arc<Self>,
        game_id: Uuid,
        color: Color,
    ) -> Result<(), &'static str> {
        self.db
            .create_game(&game_id.to_string(), &color.to_str())
            .await?;

        if self
            .games
            .write()
            .unwrap()
            .insert(game_id, Game::new(game_id, color))
            .is_some()
        {
            error!("A game with that id already exists: {}", game_id);
            return Err(INTERNAL_SERVER_ERROR);
        }
        if self
            .rooms
            .write()
            .unwrap()
            .insert(game_id, WebSocketRoom::new(game_id, Arc::clone(self)))
            .is_some()
        {
            error!("A game with that id already exists: {}", game_id);
            return Err(INTERNAL_SERVER_ERROR);
        }

        Ok(())
    }
    pub fn remove_game(&self, game_id: Uuid) {
        self.games.write().unwrap().remove(&game_id);
    }
}
