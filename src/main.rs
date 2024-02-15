use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use pool::{MicaTask, Pool};
use serde_json::json;

mod minimax;
mod pool;

use minimax::*;

type MicaBestMove = i32;

fn get_best_move(mica_request: MicaRequest, pool: Arc<Pool<MicaBestMove>>, rx: Arc<Receiver<MicaBestMove>>) -> Option<MicaMove> {
    // Arc::clone(&pool).submit(|| 0);
    let game = MicaState::from_request(mica_request);
    let moves = game.get_moves();
    for &next_move in moves.iter() {
        let mut game_clone = game.clone();
        game_clone.apply_move(next_move);
        game_clone.current_player.toggle();
        let task: MicaTask<MicaBestMove> = Box::new(move || {
            let (value, _) = game_clone.minimax(6, i32::MIN, i32::MAX);
            println!("Thread got value {value}");
            value
        });
        Arc::clone(&pool).submit(task);
    }

    let mut best_value = match game.current_player {
        MicaPlayer::White => i32::MIN,
        MicaPlayer::Black => i32::MAX,
        _ => 0,
    };
    let mut best_move = None;
    for (i, value) in rx.iter().take(moves.len()).enumerate() {
        println!("{value}");
        match game.current_player {
            MicaPlayer::White => {
                if value > best_value {
                    best_value = value;
                    best_move = Some(moves[i]);
                }
            },
            MicaPlayer::Black => {
                if value < best_value {
                    best_value = value;
                    best_move = Some(moves[i]);
                }
            },
            _ => (),
        }
    }

    // let (_, best_move) = game.minimax(6, i32::MIN, i32::MAX);
    best_move
}

fn handle_connection(mut stream: TcpStream, pool: Arc<Pool<MicaBestMove>>, rx: Arc<Receiver<MicaBestMove>>) {
    // let mut buf_reader = BufReader::new(&mut stream);
    let mut buf = [0; 1024];

    let n = stream.read(&mut buf).unwrap();

    let req = String::from_utf8_lossy(&buf[..n]);
    let request: String = req.lines().skip_while(|line| !line.is_empty()).collect();

    let mica_request: MicaRequest = serde_json::from_str(&request).unwrap();
    println!("Mica request\n{:?}", mica_request);
    let player = mica_request.player;
    
    let best_move = get_best_move(mica_request, pool, rx);

    let result = match best_move {
        None => json!({ "move": null }),
        Some(MicaMove::Set { x, y, z }) => json!({ "move": [["set", player, x, y, z]] }),
        Some(MicaMove::Move { from_x, from_y, from_z, to_x, to_y, to_z }) => json!({ "move": [["move", player, to_x, to_y, to_z, from_x, from_y, from_z]] }),
        Some(MicaMove::SetRemove { x, y, z, remove_x, remove_y, remove_z }) => {
            json!({ "move": [
                ["set", player, x, y, z],
                ["remove", player, remove_x, remove_y, remove_z]
            ]})
        },
        Some(MicaMove::MoveRemove { from_x, from_y, from_z, to_x, to_y, to_z, remove_x, remove_y, remove_z }) => {
            json!({ "move": [
                ["move",player,  to_x, to_y, to_z, from_x, from_y, from_z],
                ["remove", player, remove_x, remove_y, remove_z]
            ]})
        }
    };

    let status_line = "HTTP/1.1 200 OK";
    let contents = result.to_string();
    let length = contents.len();

    let response = format!(
        "{status_line}\r\nContent-Type: applicaton/json\r\nContent-Length: {length}\r\n\r\n{contents}"
    );
    stream.write_all(response.as_bytes()).unwrap();
}

fn main() {
    let pool = Arc::new(Pool::new());
    let rx = Arc::new(Arc::clone(&pool).init(8));
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let rx = Arc::clone(&rx);
        handle_connection(stream, Arc::clone(&pool), rx);
    }
}
