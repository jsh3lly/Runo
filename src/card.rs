use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use CardKind::*;
use Color::*;

use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;

#[derive(Debug)]
pub enum CardKind {Number, Skip, Reverse, Draw2, Draw4, Wild}

#[derive(Debug, EnumIter, Clone, Copy)]
pub enum Color {Red, Green, Blue, Yellow}


#[derive(Debug)]
pub struct Card {
    kind: CardKind,
    color: Option<Color>,
    number: Option<u8>,
}

impl Card {
    pub fn new_number(number : u8, color: Color) -> Card {
        Card {kind: CardKind::Number, color:Some(color), number:Some(number)}
    }
    pub fn new_power(kind : CardKind, color: Option<Color>) -> Card {
        match kind {
            Skip | Reverse | Draw2 => Card {kind, color, number:None},
            Draw4 | Wild => Card {kind, color:None, number:None},
            Number => panic!("Invalid kind for power card")
        }
    }
}

#[derive(Debug)]
pub struct Deck(Vec<Card>);

impl Deck {
    pub fn new() -> Deck {
        let mut deck_vec : Vec<Card> = Vec::with_capacity(112);

        for color in Color::iter() {
            deck_vec.push(Card::new_number(0, color)); // 0 card

            // 2 sets of: 1-9, draw2, skip, reverse
            for _ in 0..2 {
                for num in 1..=9 {
                    deck_vec.push(Card::new_number(num, color));
                }

                deck_vec.push(Card::new_power(Draw2, Some(color)));
                deck_vec.push(Card::new_power(Reverse, Some(color)));
                deck_vec.push(Card::new_power(Skip, Some(color)));
            }

        }

        // 4 wilds and 4 draw4s
        for _ in 0..4 {
            deck_vec.push(Card::new_power(Wild, None));
            deck_vec.push(Card::new_power(Draw4, None));
        }

        Deck(deck_vec)
    }

    pub fn pop_random_card(&mut self) -> Card {
        self.0.remove(thread_rng().gen_range(0..self.0.len()))
    }
}
