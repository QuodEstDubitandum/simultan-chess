use std::{env, sync::Arc};

use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use chess_voting::{
    game::chess_piece::Color,
    server::Server,
    utils::{
        error::INTERNAL_SERVER_ERROR,
        middleware::Authentication,
        request::{verify_jwt, FinishRequest, StartRequest},
        response::{serialize_field, GameState},
    },
    ws::WebSocketConnection,
};
use dotenv::dotenv;
use log::{error, info, warn};
// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use uuid::Uuid;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let url = env::var("URL").expect("Missing URL env var");
    let port = env::var("PORT").expect("Missing PORT env var");
    env_logger::Builder::from_default_env().init();
    info!("Server listening on port {}", port);

    // let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    // builder
    //     .set_private_key_file("certs/key.pem", SslFiletype::PEM)
    //     .unwrap();
    // builder
    //     .set_certificate_chain_file("certs/cert.pem")
    //     .unwrap();

    let server = web::Data::new(Server::new().await);
    HttpServer::new(move || {
        App::new()
            .app_data(server.clone())
            .service(health)
            .service(
                web::scope("game")
                    .wrap(Authentication {})
                    .service(get_ids)
                    .service(get_game_history)
                    .service(get_game_state)
                    .service(start_game)
                    .service(finish_game),
            )
            .service(web::scope("ws").service(connect_ws))
    })
    .bind(format!("{}:{}", url, port))?
    // .bind_openssl(format!("{}:{}", url, port), builder)?
    .run()
    .await
}

#[get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().body("OK".to_string())
}

#[get("/ids")]
async fn get_ids(req: HttpRequest, server: web::Data<Server>) -> HttpResponse {
    let jwt = req.headers().get("jwt");
    if jwt.is_none() {
        error!("No JWT provided");
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    if verify_jwt(jwt.unwrap().to_str().unwrap()).is_err() {
        error!("Incorrect JWT provided: {}", jwt.unwrap().to_str().unwrap());
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    info!("Getting all active games...");
    match server.db.get_active_games().await {
        Err(_) => HttpResponse::InternalServerError().body(INTERNAL_SERVER_ERROR),
        Ok(game) => {
            info!("Fetched ids, found {} active games", game.len());
            HttpResponse::Ok().json(game)
        }
    }
}

#[get("/{game_id}/history")]
async fn get_game_history(path: web::Path<String>, server: web::Data<Server>) -> HttpResponse {
    info!("Checking game history...");
    let game_id = path.into_inner();
    match server.db.get_moves(&game_id).await {
        Err(_) => HttpResponse::InternalServerError().body(INTERNAL_SERVER_ERROR),
        Ok(moves) => {
            info!("Fetched moves from history, got {} moves", moves.len());
            HttpResponse::Ok().json(moves)
        }
    }
}

#[get("/{game_id}/current_state")]
async fn get_game_state(path: web::Path<String>, server: web::Data<Server>) -> HttpResponse {
    info!("Checking current game state...");
    let game_id: Uuid;
    match Uuid::parse_str(&path.into_inner()) {
        Err(e) => {
            error!("Could not parse Uuid from request: {}", e);
            return HttpResponse::BadRequest().body("Bad Request");
        }
        Ok(res) => game_id = res,
    }

    let games = server.games.read().unwrap();
    let game = match games.get(&game_id) {
        None => {
            warn!("Could not find a game with id {}", game_id);
            return HttpResponse::NoContent().body("Could not find your current game");
        }
        Some(game) => game,
    };

    let game_state = GameState {
        state: serialize_field(&game.field),
        admin_color: game.admin_color.to_str(),
    };

    info!("Fetched the game state");
    HttpResponse::Ok().json(game_state)
}

#[post("/start")]
async fn start_game(req: web::Json<StartRequest>, server: web::Data<Server>) -> HttpResponse {
    info!("Starting new game...");
    let uuid = Uuid::new_v4();

    let admin_color = match req.admin_color.as_str() {
        "black" => Color::BLACK,
        "white" => Color::WHITE,
        _ => return HttpResponse::BadRequest().body("Bad Request"),
    };

    match server.add_game(uuid, admin_color).await {
        Err(_) => HttpResponse::InternalServerError().body(INTERNAL_SERVER_ERROR),
        Ok(_) => {
            info!("Created new game with id {}", uuid.to_string());
            HttpResponse::Ok().body(uuid.to_string())
        }
    }
}

#[post("/{game_id}/finish")]
async fn finish_game(
    path: web::Path<String>,
    req: web::Json<FinishRequest>,
    server: web::Data<Server>,
) -> HttpResponse {
    info!("Finishing game...");
    let game_id = match Uuid::parse_str(&path.into_inner()) {
        Err(e) => {
            error!("Could not parse Uuid from request: {}", e);
            return HttpResponse::BadRequest().body("Bad Request");
        }
        Ok(id) => id,
    };

    server.remove_game(game_id);
    match server
        .db
        .finish_game(&req.game_result, &game_id.to_string())
        .await
    {
        Err(_) => HttpResponse::InternalServerError().body(INTERNAL_SERVER_ERROR),
        Ok(_) => {
            info!("Finished DB game with result {:?}", req.game_result);
            HttpResponse::Ok().body("OK".to_string())
        }
    }
}

#[get("/connect/{game_id}")]
async fn connect_ws(
    path: web::Path<String>,
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Server>,
) -> HttpResponse {
    let game_id = match Uuid::parse_str(&path.into_inner()) {
        Err(e) => {
            error!("Could not parse Uuid from request: {}", e);
            return HttpResponse::BadRequest().body("Bad Request");
        }
        Ok(res) => res,
    };
    let connection = WebSocketConnection::new(game_id, Arc::clone(&server));
    let resp = ws::start(connection, &req, stream);
    if let Err(e) = resp {
        error!("Could not connect to websocket: {}", e);
        return HttpResponse::InternalServerError().body("Internal server error");
    }

    resp.unwrap()
}
