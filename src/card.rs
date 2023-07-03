use core::fmt;

use strum::IntoEnumIterator;
use strum_macros::{EnumIter, Display};
use CardKind::*;
use colored::*;

use rand::{thread_rng, Rng};

use serde::{Serialize, Deserialize};

#[derive(Debug, Display, Serialize, Deserialize, Clone, PartialEq)]
pub enum CardKind {Number, Skip, Reverse, Draw2, Draw4, Wild}

#[derive(Debug, Display, EnumIter, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Color {Red, Green, Blue, Yellow}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Card {
    pub kind: CardKind,
    pub color: Option<Color>,
    pub number: Option<u8>,
}

impl Card {
    fn new_number(number : u8, color: Color) -> Card {
        Card {kind: CardKind::Number, color:Some(color), number:Some(number)}
    }
    fn new_power(kind : CardKind, color: Option<Color>) -> Card {
        match kind {
            Skip | Reverse | Draw2 => Card {kind, color, number:None},
            Draw4 | Wild => Card {kind, color:None, number:None},
            Number => panic!("Invalid kind for power card")
        }
    }
    pub fn set_draw4_or_wild_color(&mut self, color: Color) {
        match self.kind {
            Draw4 | Wild => self.color = Some(color),
            _ => panic!("This should not happen!!"),
        }
    }

    pub fn get_colorized_repr(&self) -> String {
        let color_str = match self.color {
            Some(Color::Red) => "red",
            Some(Color::Green) => "green",
            Some(Color::Blue) => "blue",
            Some(Color::Yellow) => "yellow",
            None => "cyan"
        };
        self.to_string().color(color_str).to_string()
    }
}

// fn result_transmuter(res: io::Result) -> fmt::Result {
//
// }

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.color {
            Some(_) => {
                match self.kind {
                    CardKind::Number => {write!(f, "{} {}", self.color.unwrap(), self.number.unwrap())}
                    _ => {write!(f, "{} {}", self.color.unwrap(), self.kind)}
                }
            }
            None => {f.write_str(&format!("{}", self.kind).black())}
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
        // Card::new_number(1, Color::Green)
        self.0.remove(thread_rng().gen_range(0..self.0.len()))
    }

    pub fn push_card(&mut self, card: Card) {
        self.0.push(card);
    }

}

#[derive(Serialize, Deserialize, Clone)]
pub struct Hand(Vec<Card>);
impl Hand {
    pub fn new(init_hand_size : usize, deck : &mut Deck) -> Hand {
        let mut cards : Vec<Card> = vec![];
        // cards.push(Card::new_number(2, Color::Green));
        // cards.push(Card::new_number(3, Color::Green));
        (0..init_hand_size).for_each(|_| cards.push(deck.pop_random_card()));
        Hand(cards)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    //TODO: validate bounds
    pub fn pop_at(&mut self, index : usize) -> Card {
        self.0.remove(index - 1)
    }

    pub fn get_at(&self, index: usize) -> Card {
        self.0[index - 1].clone()
    }

    pub fn push(&mut self, card: Card) {
        self.0.push(card);
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Your hand is:").unwrap();
        for i in 0..self.0.len() {
            let color_str = match self.0[i].color {
                Some(color) => {
                    match color {
                        Color::Red => "red",
                        Color::Green => "green",
                        Color::Blue => "blue",
                        Color::Yellow => "yellow"
                    }
                }
                None => "cyan",
            };
            let uncolored_str = format!("[{}]  {}\n", i+1, self.0[i]);
            f.write_str(&format!("{}", uncolored_str.color(color_str))).unwrap();
        }
        Ok(())
    }
}

impl fmt::Debug for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Hand details hidden for obvious reasons ;)").unwrap();
        Ok(())
    }
}
