// STD
use std::any::{Any, TypeId};
use std::fmt;
use std::rc::Rc;

// UU
use crate::cards::*;

pub type Actions = Vec<Action>;
pub type History = Vec<Rc<Action>>;

macro_rules! add_cards {
    ($deck:expr, $cls:ident, $num:expr ) => {
        for _ in 0..$num {
            $deck.push(Box::new($cls {}));
        }
    };
}

#[derive(Debug, Clone)]
pub struct Player {
    pub hand: Cards,
    pub stable: Cards
}

impl Player {
    fn new() -> Player {
        return Player {
            hand: Vec::new(),
            stable: Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub players: Vec<Player>,
    pub deck: Cards,
    pub nursery: Cards,
    pub discard: Cards
}

impl Board {
    pub fn new_base_game(player_count: u8) -> Board {
        let mut deck: Cards = Vec::new();

        // Add number of cards
        add_cards!(deck, BasicUnicorn, 22);
        add_cards!(deck, Neigh, 3);
        add_cards!(deck, SuperNeigh, 1);

        assert!(player_count >= 2, "Must have atleast two players.");

        let mut players = Vec::new();
        for i in 0..player_count {
            players.push(Player::new())
        }

        let mut nursery: Cards = Vec::new();
        add_cards!(nursery, BabyUnicorn, 13);
        let board = Board {
            players,
            deck,
            nursery,
            discard: Vec::new()
        };

        return  board;
    }

    /// Draws a specified card if applicable.
    pub fn draw_specific_card<T: 'static + Card>(&self) -> Option<Action> {
        let (c, new_deck) = self.deck.remove_one_card_with_type::<T>()?;
        if new_deck.len() != self.deck.len() {
            let new_board = Board {
                players: self.players.clone(),
                deck: new_deck.clone(),
                nursery: self.nursery.clone(),
                discard: self.discard.clone()
            };

            return Some(Action {
                card: c.clone(),
                atype: ActionType::Draw,
                board: new_board
            });
        }
        return None;
    }

}

#[derive(Debug)]
pub struct GameState {
    pub board: Board,
    pub phase: PhaseType
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    Place,
    Instant,
    Steal,
    Destroy,
    Stolen,
    Discard,
    Draw
}

#[derive(Debug, Clone, PartialEq)]
pub enum PhaseType {
    GameStart,
    Play,
    Effect,
    Turn,
    React,
    Draw
}

#[derive(Debug, Clone)]
pub struct Action {
    pub card: Box<dyn Card>,
    pub atype: ActionType,
    pub board: Board
}

mod StateTest {
    use super::*;

    #[test]
    fn test_board_draw() {
        let drawn_card = Board::new_base_game(2)
                            .draw_specific_card::<Neigh>()
                            .unwrap()
                            .card;
        assert!(drawn_card.name() == "Neigh", "Drawn deck should match.")
    }
}
