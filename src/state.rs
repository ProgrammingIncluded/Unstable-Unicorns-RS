// STD
use std::any::{Any, TypeId};
use std::fmt;

// UU
use crate::cards::*;

pub type Actions = Vec<Action>;

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
    pub discard: Cards
}

impl Board {
    pub fn new_base_game(player_count: u8) -> Board {
        let deck: Cards = vec![
            Box::new(Neigh {}),
            Box::new(Neigh {}),
            Box::new(Neigh {}),
            Box::new(SuperNeigh {})
        ];

        assert!(player_count >= 2, "Must have atleast two players.");

        let mut players = Vec::new();
        for i in 0..player_count {
            players.push(Player::new())
        }

        let board = Board {
            players: players,
            deck: deck, 
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

#[derive(Debug, Clone)]
pub enum ActionType {
    Place,
    Instant,
    Steal,
    Destroy,
    Stolen,
    Discard,
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
        let drawn_card = Board::new_base_game(2).draw_specific_card::<Neigh>().unwrap().card;
        assert!(drawn_card.name() == "Neigh", "Drawn deck should match.")
    }
}