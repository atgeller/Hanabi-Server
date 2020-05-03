extern crate rand;
extern crate rustc_serialize;

use std::fmt;
use std::collections::VecDeque;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Copy, Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
pub enum Color {
    White = 0,
    Yellow = 1,
    Red = 2,
    Blue = 3,
    Green = 4,
    Rainbow = 5,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Color::White => write!(f, "White"),
            Color::Yellow => write!(f, "Yellow"),
            Color::Red => write!(f, "Red"),
            Color::Blue => write!(f, "Blue"),
            Color::Green => write!(f, "Green"),
            Color::Rainbow => write!(f, "Rainbow"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
pub enum Value {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::One => write!(f, "One"),
            Value::Two => write!(f, "Two"),
            Value::Three => write!(f, "Three"),
            Value::Four => write!(f, "Four"),
            Value::Five => write!(f, "Five"),
        }
    }
}

const COLORS : [Color; 6] = [Color::White, Color::Yellow, Color::Red, Color::Blue, Color::Green, Color::Rainbow];
const VALUES: [Value; 5] = [Value::One,Value::Two,Value::Three,Value::Four,Value::Five];

#[derive(Copy, Clone, Debug, RustcDecodable, RustcEncodable)]
pub struct Card { pub color: Color, pub value: Value }

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.color, self.value)
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Deck(VecDeque<Card>);

impl fmt::Display for Deck {
    // From Rust by example
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vec = &self.0;

        write!(f, "[")?;

        for (count, v) in vec.iter().enumerate() {
            if count != 0 { write!(f, ", ")?; }
            write!(f, "{}", v)?;
        }

        write!(f, "]")
    }
}

pub fn init_count(val: Value) -> i32 {
    match val {
        Value::One => 3,
        Value::Two | Value::Three | Value::Four => 2,
        Value::Five => 1,
    }
}

impl Deck {
    const DECKSIZE: usize = 60;

    pub fn initialize(include_rainbow: bool) -> Deck {
        let mut cards = VecDeque::<Card>::with_capacity(Deck::DECKSIZE);
        for color in COLORS.iter() {
            if !include_rainbow && *color == Color::Rainbow {
                continue;
            }

            for val in VALUES.iter() {
                for _i in 0..init_count(*val) {
                    cards.push_back(Card{color: *color, value: *val});
                }
            }
        }
    
        Deck(cards)
    }
    
    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        let mut vec = &mut self.0;
        let mut cards = vec.as_mut_slices().0;
        cards.shuffle(&mut rng);
    }

    pub fn draw(&mut self) -> Option<Card> {
        let mut vec = &mut self.0;
        vec.pop_front()
    }
}