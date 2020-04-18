mod HanabiLib;
use HanabiLib::Game as Game;
use Game::Action as Action;
use HanabiLib::Deck as Deck;

extern crate rustc_serialize;
use rustc_serialize::json;

use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::fs;

use std::vec::Vec;

const ROOM_SIZE : usize = 3;

fn main() {    
    let mut game = Game::Game::new(ROOM_SIZE);

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let mut streams = Vec::<TcpStream>::with_capacity(ROOM_SIZE);
    let mut players = 0;

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        players = handle_connect(stream, &game, players);
    }

    // EVENT LOOP
    /*
    while !game.is_over() {
        for i in 0..ROOM_SIZE {
            handle_event(&streams[i]);
        }

        std::thread::sleep(std::time::Duration::from_millis(10))
    }*/
}

fn handle_connect(mut stream: TcpStream, mut game : &Game::Game, players: i32) -> i32 {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);
    println!("Request: {}", request);

    if request.starts_with("GET") {
        let response = handle_get(request.into_owned(), game, players);
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        return players;
    } else if request.starts_with("POST") {
        let _updated = handle_post(request.into_owned(), game, players);
        // TODO: if state updated send new state to participants
        return players;
    } else {
        // BAD REQUEST
        return players;
    }
}

fn make_HTTP_respone(head: String, header: Vec::<String>, contents: String) -> String {
    let accumulated = header.iter().fold(String::from(""), |acc, x| format!("{}{}\r\n", acc, x));
    format!("HTTP/1.1 {}\r\n{}\r\n{}", head, accumulated, contents)
}

fn handle_get(request: String, game : &Game::Game, players: i32) -> String {
    let mut response = String::from("HTTP/1.1 404 STUPID BROWSER\r\n\r\n");

    let start = request.find("/");
    let end = request.find(" HTTP/1.1");

    if start.is_none() || end.is_none() {
        response = make_HTTP_respone(response, Vec::<String>::new(), String::new());
    } else {
        let requested = request.get(start.unwrap()+1..end.unwrap()).unwrap();
        println!("{}", requested);

        response = match requested {
            "" => {
                make_HTTP_respone(String::from("200 OK"), vec![String::from("Content-Type: text/html"), format!("Set-Cookie: player={}", players)], fs::read_to_string("enter.html").unwrap())
            },

            "cards.js" => make_HTTP_respone(String::from("200 OK"), vec![String::from("Content-Type: text/javascript")], fs::read_to_string("cards.js").unwrap()),

            "style.css" => make_HTTP_respone(String::from("200 OK"), vec![String::from("Content-Type: text/css")], fs::read_to_string("style.css").unwrap()),

            _ => response,
        }
    }

    response
}

fn handle_post(request: String, mut game : &Game::Game, players: i32) -> String {
    let action = "POST / HTTP/1.1\r\n";

    if request.starts_with(action) {
        // if request.find("Action: ") {}
        println!("Action Message: {:?}", request.find("Action-Type: "));
    }

    String::new()
}
