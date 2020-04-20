extern crate actix;
extern crate actix_web_actors;

use actix::prelude::*;
use actix_web_actors::ws;

use std::fs;

pub mod hanabi_lib;
use hanabi_lib::game::{Game,Action,Hint};

use std::vec::Vec;
use std::collections::HashMap;

use rand::{self, rngs::ThreadRng, Rng};

extern crate rustc_serialize;
use rustc_serialize::json;

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct Join {
    pub addr: Recipient<Message>,
    pub name: String,
}

#[derive(Message)]
#[rtype(String)]
pub struct PlayCard {
    pub id: usize,
    pub card_index: usize,
}

#[derive(Message)]
#[rtype(String)]
pub struct Discard {
    pub id: usize,
    pub card_index: usize,
}

#[derive(Message)]
#[rtype(String)]
pub struct GiveHint {
    pub id: usize,
    pub other_player: String,
    pub hint: String,
}

#[derive(Message)]
#[rtype(String)]
pub struct Swap {
    pub id: usize,
    pub index1: usize,
    pub index2: usize,
}

struct ConnectedPlayer {
    name: String,
    id: usize,
    address: Recipient<Message>,
}

/// Define http actor
pub struct GameServer {
    players: Vec<ConnectedPlayer>,
    ids: HashMap<usize,usize>,
    rng: ThreadRng,
    game: Game,
    max_players: usize,
}

impl GameServer {
    pub fn new(max_players: usize) -> GameServer {
        GameServer {
            players: Vec::<ConnectedPlayer>::with_capacity(max_players),
            ids: HashMap::<usize,usize>::new(),
            rng: rand::thread_rng(),
            game: Game::new(max_players),
            max_players: max_players,
        }
    }

    fn send_update(&mut self) {
        for i in 0..self.players.len() {
            self.players[i].address.do_send(Message(self.game.get_player_view(i)));
        }
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;
}

impl Handler<Join> for GameServer {
    type Result = usize;

    fn handle(&mut self, msg: Join, _: &mut Self::Context) -> Self::Result {
        if self.players.len() >= self.max_players {
            return 0;
        }

        let id = self.rng.gen_range(1, std::usize::MAX);
        self.ids.insert(id, self.players.len());
        self.players.push(ConnectedPlayer {
            name: msg.name.clone(),
            id: id,
            address: msg.addr,
        });

        self.game.set_name(self.players.len() - 1, msg.name.clone());

        println!("Player {} joined as {} with id {}", msg.name, self.players.len() - 1, id);

        if (self.players.len() == self.max_players) {
            self.send_update();
        }

        id
    }
}

impl Handler<PlayCard> for GameServer {
    type Result = String;

    fn handle(&mut self, msg: PlayCard, _: &mut Self::Context) -> Self::Result {
        if self.players.len() < self.max_players {
            return String::from("ERROR: The Game hasn't started yet!!!");
        }

        let player_index = match self.ids.get(&msg.id) {
            Some(index) => index,
            None => return String::from("ERROR: ID Invalid!")
        };
        
        println!("Player #{} playing {}", player_index, msg.card_index);

        let valid = self.game.take_action(Action::Play(msg.card_index),*player_index);

        if !valid {
            String::from("ERROR: Invalid Action!!!")
        } else {
            self.send_update();
            String::from("Success")
        }
    }
}

impl Handler<Discard> for GameServer {
    type Result = String;

    fn handle(&mut self, msg: Discard, _: &mut Self::Context) -> Self::Result {
        if self.players.len() < self.max_players {
            return String::from("ERROR: The Game hasn't started yet!!!");
        }

        let player_index = match self.ids.get(&msg.id) {
            Some(index) => index,
            None => return String::from("ERROR: ID Invalid!")
        };
        
        println!("Player #{} discarding {}", player_index, msg.card_index);

        let valid = self.game.take_action(Action::Discard(msg.card_index),*player_index);

        if !valid {
            return String::from("ERROR: Invalid Action!!!");
        } else {
            self.send_update();
            String::from("Success")
        }
    }
}

impl Handler<Swap> for GameServer {
    type Result = String;

    fn handle(&mut self, msg: Swap, _: &mut Self::Context) -> Self::Result {
        if self.players.len() < self.max_players {
            return String::from("ERROR: The Game hasn't started yet!!!");
        }

        let player_index = match self.ids.get(&msg.id) {
            Some(index) => index,
            None => return String::from("ERROR: ID Invalid!")
        };
        
        println!("Player #{} swapping {} and {}", player_index, msg.index1, msg.index2);

        let valid = self.game.take_action(Action::Swap(msg.index1, msg.index2),*player_index);

        if !valid {
            return String::from("ERROR: Invalid Action!!!");
        } else {
            self.send_update();
            String::from("Success")
        }
    }
}

impl Handler<GiveHint> for GameServer {
    type Result = String;

    fn handle(&mut self, msg: GiveHint, _: &mut Self::Context) -> Self::Result {
        if self.players.len() < self.max_players {
            return String::from("ERROR: The Game hasn't started yet!!!");
        }

        let player_index = match self.ids.get(&msg.id) {
            Some(index) => index,
            None => return String::from("ERROR: ID Invalid!")
        };

        println!("{}", &msg.hint);

        let hint: Hint = json::decode::<Hint>(&msg.hint).unwrap();

        let mut other_player_index = self.players.len();
        for i in 0..self.players.len() {
            if self.players[i].name == msg.other_player {
                other_player_index = i;
                break
            }
        }
        
        if other_player_index == self.players.len() {
            return String::from("ERROR: Other player not found!")
        }

        println!("Player #{} giving hint {:?} to {}", player_index, hint, other_player_index);

        let valid = self.game.take_action(Action::GiveHint(hint,other_player_index), *player_index,);

        if !valid {
            String::from("ERROR: Invalid Action!!!")
        } else {
            self.send_update();
            String::from("Success")
        }
    }
}
