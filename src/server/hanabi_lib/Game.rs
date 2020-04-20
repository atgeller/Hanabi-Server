use std::fmt;
use std::collections::VecDeque;
use std::vec::Vec;
extern crate rustc_serialize;

use rustc_serialize::json;

use crate::server::hanabi_lib::deck::{init_count, Value, Color, Card, Deck};

#[derive(Clone, Copy, Debug, RustcDecodable, RustcEncodable)]
pub enum Hint {
    ColorHint(Color),
    ValueHint(Value),
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct Player {
    name : String,
    known_colors : [Option::<Color>; 5],
    known_values : [Option::<Value>; 5],
    hand : VecDeque<Card>,
}

#[derive(Clone, Copy, Debug, RustcDecodable, RustcEncodable)]
pub enum Action {
    Discard(usize),
    Play(usize),
    GiveHint(Hint, usize),
    Swap(usize, usize),
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Game {
    deck:        Deck,
    discard:     VecDeque<Card>,
    hints_left:  usize,
    bombs:       usize,
    turn:        usize,
    players:     Vec<Player>,
    piles:       [usize; 6],
    state:       Status,
    last_action: Option<String>,
}

#[derive(Clone, Copy, Debug, RustcDecodable, RustcEncodable)]
enum Status {
    Won,
    Lost,
    Playing,
}

#[derive(Clone, Copy, Debug, RustcDecodable, RustcEncodable)]
struct CardView {
    color : Option<Color>,
    value : Option<Value>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct PlayerView {
    name : String,
    cards : [CardView; 5],
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct GameView {
    discard:    VecDeque<Card>,
    hints_left: usize,
    bombs:      usize,
    turn:       usize,
    players:    Vec<PlayerView>,
    piles:      [usize; 6],
    state:      Status,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct Update {
    view: GameView,
    action: Option<String>,
}

impl Game {
    pub fn new(num_players: usize) -> Game {
        let mut deck = Deck::initialize();
        for _ in 0..7 {
            deck.shuffle();
        }

        let mut players : Vec::<Player> = Vec::with_capacity(num_players);
        for player in 0..num_players {
            let mut hand : VecDeque<Card> = VecDeque::with_capacity(5);
            
            for y in 0..5 {
                // SURE AS FUCK WON'T PANIC OR WAY TOO FUCKING MANY PLAYERS WHAT THE FUCK IS YOUR INPUT VALIDATION DOING YOU FUCKING JOKE OF A HUMAN?
                hand.push_back(deck.draw().unwrap());
            }
            
            players.push(Player {
                name : String::new(),
                known_colors : [None; 5],
                known_values : [None; 5],
                hand: hand,
            });
        }

        Game{
            deck: deck,
            discard: VecDeque::new(),
            hints_left: 8,
            bombs: 0,
            turn: 0,
            players: players,
            piles: [0; 6],
            state: Status::Playing,
            last_action: None,
        }
    }

    pub fn set_name(&mut self, player: usize, name: String) {
        self.players[player].name = name;
    }

    fn draw(&mut self, actor: usize) {
        let mut hand = &mut self.players[actor as usize].hand;
        let drawn = self.deck.draw();
        if drawn.is_some() {
            hand.push_back(drawn.unwrap());
        }
    }

    fn use_card(&mut self, actor: usize, card_index: usize) -> Card {
        let mut player = &mut self.players[actor as usize];
        for i in card_index..player.hand.len()-1 {
            player.known_colors[i] = player.known_colors[i + 1];
            player.known_values[i] = player.known_values[i + 1];
        }

        player.known_colors[player.hand.len()-1] = None;
        player.known_values[player.hand.len()-1] = None;

        player.hand.remove(card_index).unwrap()
    }

    fn update_state(&mut self, discarded : Option<Card>) {
        if self.piles.iter().all(|&v| v == 5) {
            self.state = Status::Won;
            return;
        }

        if let Some(card) = discarded {
            let mut count = init_count(card.value);

            for other_card in self.discard.iter() {
                if card.color == other_card.color && card.value == other_card.value {
                    count = count - 1;
                }
            }

            if count == 0 {
                self.state = Status::Lost
            }
        }
        
        if self.bombs == 3 {
            self.state = Status::Lost;
        }
        
        if self.players.iter().all(|p| p.hand.len() < 5) {
            self.state = Status::Lost;
        }
    }

    pub fn is_over(&mut self) -> bool {
        match self.state {
            Status::Won => true,
            Status::Lost => true,
            Status::Playing => false,
        }
    }

    fn give_hint(&mut self, hint: Hint, receiver: usize) {
        let mut player = &mut self.players[receiver];
        let hand = &player.hand;

        match hint {
            Hint::ColorHint(color) => {
                let mut known_colors = &mut player.known_colors;
                for i in 0..hand.len() {
                    if hand[i].color == color {
                        known_colors[i] = Some(color);
                    }
                }
            },

            Hint::ValueHint(value) => {
                let mut known_values = &mut player.known_values;
                for i in 0..hand.len() {
                    if hand[i].value == value {
                        known_values[i] = Some(value);
                    }
                }
            },
        }
    }

    pub fn take_action(&mut self, action: Action, actor: usize) -> bool {
        if self.is_over() {
            println!("The game has ended!");
            return false;
        }

        if actor != self.turn {
            if let Action::Swap(_,_) = action {} else {
                println!("NOT PLAYER {}'s TURN!!!", actor);
                return false;
            }
        }

        let actor_name = self.players[actor].name.clone();

        match action {
            Action::Discard(index) => {
                let mut player = &mut self.players[actor];
                let mut hand = &mut player.hand;

                if index >= hand.len() {
                    println!("No Card there to play!");
                    return false;
                }

                let card = self.use_card(actor, index);
                self.discard.push_back(card);
                self.draw(actor);
                self.hints_left = std::cmp::min(self.hints_left + 1, 8);
                self.last_action = Some(format!("{} discarded {} {}", actor_name, card.color, card.value));

                self.update_state(Some(card));
            },

            Action::Play(index) => {
                let mut player = &mut self.players[actor];
                let mut hand = &mut player.hand;

                if index >= hand.len() {
                    println!("No Card there to play!");
                    return false;
                }

                let card = self.use_card(actor, index);
                
                if self.piles[card.color as usize] + 1 == card.value as usize {
                    self.piles[card.color as usize] = card.value as usize;
                    if card.value == Value::Five {
                        self.hints_left = std::cmp::min(self.hints_left + 1, 8);
                    }

                    self.draw(actor);
                    self.update_state(None);
                } else {
                    self.discard.push_back(card);
                    self.bombs = self.bombs + 1;

                    self.draw(actor);
                    self.update_state(Some(card));
                }

                self.last_action = Some(format!("{} played {} {}", actor_name, card.color, card.value));
            },

            Action::GiveHint(hint, other_player) => {
                if (self.hints_left == 0) {
                    println!("NO HINTS TO GIVE");
                    return false;
                }

                self.give_hint(hint, other_player);
                self.hints_left = self.hints_left - 1;

                let hint_string = match hint {
                    Hint::ColorHint(color) => format!("color {:?}", color),
                    Hint::ValueHint(value) => format!("value {:?}", value),
                };
    
                self.last_action = Some(format!("{} gave a hint about cards with the {} to {}", actor_name, hint_string, self.players[other_player].name.clone()));
            },

            Action::Swap(index1, index2) => {
                let mut player = &mut self.players[actor];

                if index1 >= player.hand.len() || index2 >= player.hand.len() {
                    return false;
                }                

                player.hand.swap(index1, index2);
                player.known_colors.swap(index1, index2);
                player.known_values.swap(index1, index2);
            }
        }

        self.turn = (self.turn + 1) % self.players.len();
        return true;
    }

    pub fn get_player_view(&self, actor: usize) -> String {
        let mut views = Vec::<PlayerView>::with_capacity(self.players.len());

        for i in 0..self.players.len() {
            let player = &self.players[i];
            let mut view = PlayerView {
                name: player.name.clone(),
                cards : [CardView {
                    color: None,
                    value: None,
                }; 5],
            };

            for j in 0..5 {
                if (actor == i) {
                    view.cards[j] = CardView {
                        color: player.known_colors[j],
                        value: player.known_values[j],
                    };
                } else {
                    view.cards[j] = CardView {
                        color: Some(player.hand[j].color),
                        value: Some(player.hand[j].value),
                    };
                }
            }

            views.push(view);
        }

        let view = GameView{
            discard:    self.discard.clone(),
            hints_left: self.hints_left,
            bombs:      self.bombs,
            turn:       self.turn,
            players:    views,
            piles:      self.piles.clone(),
            state:      self.state,
        };

        json::encode(&Update{
            view: view,
            action: self.last_action.clone(),
        }).unwrap()
    }
}
