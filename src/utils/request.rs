use std::env;

use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct MoveRequest {
    pub from: String,
    pub to: String,
    pub promotion: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MoveResponse {
    pub player: String,
    pub from: String,
    pub to: String,
    pub promotion: String,
    pub en_passant: bool,
    pub result: Option<String>,
    pub move_notation: String,
}

#[derive(Deserialize, Debug)]
pub struct FinishRequest {
    pub game_result: String,
}

#[derive(Deserialize, Debug)]
pub struct StartRequest {
    pub admin_color: String,
}

#[derive(Deserialize, Debug)]
pub struct Claims {
    pub email: String,
}

pub fn verify_jwt(raw_jwt: &str) -> Result<(), &'static str> {
    let secret = env::var("JWT_SECRET").expect("Missing JWT_SECRET env var");
    let header = decode_header(raw_jwt).map_err(|_| "Invalid JWT header")?;

    let mut validation = Validation::new(header.alg);
    validation.validate_exp = true;

    let token_result = decode::<Claims>(
        raw_jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    );

    match token_result {
        Ok(data) => {
            if data.claims.email != "d.steblin.dev@gmail.com" {
                error!("Invalid JWT email address: {}", data.claims.email);
                return Err("Invalid JWT email address");
            }
            Ok(())
        }
        Err(err) => {
            error!("Invalid JWT token: {}", err);
            return Err("Invalid JWT token");
        }
    }
}
