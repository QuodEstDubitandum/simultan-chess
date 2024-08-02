use std::{env, time::SystemTime};

use actix_web::cookie::time::{format_description::well_known::Rfc3339, OffsetDateTime};
use libsql::{de, params, Builder, Connection};
use log::error;
use serde::{Deserialize, Serialize};

use crate::utils::error::INTERNAL_SERVER_ERROR;

pub async fn connect_db() -> Connection {
    let url = env::var("TURSO_DATABASE_URL").expect("TURSO_DATABASE_URL not set");
    let token = env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH_TOKEN not set");
    let env = env::var("ENV").expect("DEV not set");

    let conn: Connection;
    if env == "dev" {
        let db = Builder::new_local("local.db")
            .build()
            .await
            .expect("Could not connect to local database");
        conn = db.connect().unwrap();
    } else {
        let db = Builder::new_remote(url, token)
            .build()
            .await
            .expect("Could not connect to prod database");
        conn = db.connect().unwrap();
    }

    conn
}

pub async fn seed_db(db: &Connection) {
    db.execute_batch(
        r#"
    DROP TABLE IF EXISTS Move;
    DROP TABLE IF EXISTS Game;

    CREATE TABLE IF NOT EXISTS Game(
    game_id VARCHAR(255) PRIMARY KEY,
    admin_color VARCHAR(10),
    result VARCHAR(20),
    created_at TEXT
    );

    CREATE TABLE IF NOT EXISTS Move(
    move_id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id VARCHAR(255),
    turn INTEGER,
    player VARCHAR(10),
    move_notation VARCHAR(10),
    created_at TEXT,
    FOREIGN KEY(game_id) REFERENCES Game(game_id)
    );
    "#,
    )
    .await
    .expect("Cant seed DB");
}

pub struct DB {
    pub conn: Connection,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Move {
    pub move_notation: String,
    pub turn: u32,
    pub player: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DBGame {
    game_id: String,
    created_at: String,
    last_moved: Option<String>,
    admin_color: String,
}

impl DB {
    pub async fn new() -> DB {
        let conn = connect_db().await;
        seed_db(&conn).await;
        DB { conn }
    }
    pub async fn create_game(&self, id: &str, color: &str) -> Result<(), &'static str> {
        let system_time = SystemTime::now();
        let datetime = OffsetDateTime::from(system_time);
        let now_str = datetime.format(&Rfc3339).unwrap();
        match self
            .conn
            .execute(
                "INSERT INTO Game(game_id, admin_color, result, created_at) VALUES(?1, ?2, null, ?3)",
                params![id, color, now_str],
            )
            .await
        {
            Err(e) => {
                error!("Could not add game {} to DB: {}", id, e);
                Err(INTERNAL_SERVER_ERROR)
            }
            Ok(_) => Ok(()),
        }
    }
    pub async fn finish_game(&self, result: &str, id: &str) -> Result<(), &'static str> {
        match self
            .conn
            .execute(
                "UPDATE Game SET result = ?1 WHERE game_id = ?2",
                params![result, id],
            )
            .await
        {
            Err(e) => {
                error!("Could not finish game {} in DB: {}", id, e);
                Err(INTERNAL_SERVER_ERROR)
            }
            Ok(_) => Ok(()),
        }
    }
    pub async fn insert_move(
        &self,
        turn: u32,
        id: &str,
        new_move: &str,
        player: &str,
    ) -> Result<(), &'static str> {
        let system_time = SystemTime::now();
        let datetime = OffsetDateTime::from(system_time);
        let now_str = datetime.format(&Rfc3339).unwrap();
        match self
            .conn
            .execute(
                "INSERT INTO Move(turn, move_notation, player, game_id, created_at) VALUES(?1, ?2, ?3, ?4, ?5)",
                params![turn, new_move, player, id, now_str],
            )
            .await
        {
            Err(e) => {
                error!(
                    "Could not insert move {} in game {} into DB: {}",
                    new_move, id, e
                );
                Err(INTERNAL_SERVER_ERROR)
            }
            Ok(_) => Ok(()),
        }
    }
    pub async fn get_moves(&self, id: &str) -> Result<Vec<Move>, &'static str> {
        let rows = self
            .conn
            .query(
                "SELECT * FROM Move WHERE game_id = ?1 
                ORDER BY created_at",
                params![id],
            )
            .await;

        if let Err(e) = rows {
            error!("Could not get moves from game {}: {}", id, e);
            return Err(INTERNAL_SERVER_ERROR);
        }
        let mut rows = rows.unwrap();

        let mut moves: Vec<Move> = vec![];
        while let Some(row) = rows.next().await.unwrap() {
            moves.push(de::from_row::<Move>(&row).unwrap());
        }

        Ok(moves)
    }
    pub async fn get_active_games(&self) -> Result<Vec<DBGame>, &'static str> {
        let games = self
            .conn
            .query(
                "SELECT G.game_id AS game_id, G.admin_color, G.created_at, M.player AS last_moved
                FROM Game G
                LEFT JOIN (
                    SELECT game_id, player
                    FROM Move
                    WHERE (game_id, created_at) IN (
                        SELECT game_id, MAX(created_at)
                        FROM Move
                        GROUP BY game_id
                    )
                ) M ON G.game_id = M.game_id
                WHERE G.result IS NULL",
                (),
            )
            .await;

        if let Err(e) = games {
            error!("Could not get active games from DB: {}", e);
            return Err(INTERNAL_SERVER_ERROR);
        }
        let mut games = games.unwrap();

        let mut game: Vec<DBGame> = vec![];
        while let Some(row) = games.next().await.unwrap() {
            game.push(de::from_row::<DBGame>(&row).unwrap());
        }

        Ok(game)
    }
}
