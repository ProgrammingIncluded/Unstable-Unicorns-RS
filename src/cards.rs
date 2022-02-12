// STD
use std::any::{Any, TypeId};
use std::fmt;
use std::fmt::Debug;

// EXT
use dyn_clone::DynClone;

// UU
use crate::state::{Actions, Action, ActionType, Board};

#[derive(Debug, Clone)]
enum CardType {
    Instant,
    Magic,
    Downgrade,
    Upgrade,
    BasicUnicorn,
    MagicUnicorn,
    BabyUnicorn
}

impl CardType {
    fn is_unicorn(&self) -> bool {
        match self {
            CardType::BasicUnicorn => { true },
            CardType::MagicUnicorn => { true },
            CardType::BabyUnicorn => { true },
            _ => { false }
        }
    }
}

pub trait Card: Debug + DynClone {
    fn ctype(&self) -> CardType;
    fn name(&self) -> &'static str;
    fn play(self: Box<Self>, player: usize, history: &Actions) -> Option<Actions> { None }
    fn react(&self, player: usize, history: &Actions) -> Option<Actions> { None }
    fn destroy(&self, player: usize, history: &Actions) -> Option<Actions> { None }
    fn steal(&self, player: usize, history: &Actions) -> Option<Actions> { None }

    // For dynamic downcast
    fn as_any(&self) -> &dyn Any;

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.name())
    }
}

dyn_clone::clone_trait_object!(Card);

pub type Cards = Vec<Box<dyn Card>>;
type CardItem = Box<dyn Card>;
pub trait QueryCards {

    /// Remove Card from cards.
    fn remove_one_card_with_type<T: 'static + Card>(&self) -> Option<(CardItem, Cards)>;

    /// Check if card type exists
    fn has_card<T: 'static + Card>(&self) -> bool;

    /// Counts cards of specific type.
    fn count_card<T: 'static + Card>(&self) -> usize;
}

impl QueryCards for Cards {
    fn remove_one_card_with_type<T: 'static + Card>(&self) -> Option<(Box<dyn Card>, Cards)> {
        for (idx, c) in self.iter().enumerate() {
           if c.as_any().is::<T>() {
                let mut new_qc = self.clone();
                new_qc.remove(idx);
                return Some((c.clone(), new_qc));
            }
        }
        return None;
    }

    fn has_card<T: 'static + Card>(&self) -> bool {
        return self.iter().any(|x| x.as_any().is::<T>());
    }

    fn count_card<T: 'static + Card>(&self) -> usize {
        return self.iter().filter(|x| x.as_any().is::<T>()).count();
    }
}

#[derive(Debug, Clone)]
pub struct SuperNeigh {}
impl Card for SuperNeigh {
    fn ctype(&self) -> CardType { CardType::Instant }
    fn name(&self) -> &'static str { "SuperNeigh" }
    fn play(self: Box<Self>, player: usize, history: &Actions) -> Option<Actions> {
        return Some(vec![
            Action {
                card: self,
                atype: ActionType::Instant,
                board: history[0].board.clone(),
            }, history[0].clone()
        ]);
    }

    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct Neigh {}
impl Card for Neigh {
    fn ctype(&self) -> CardType { CardType::Instant }
    fn name(&self) -> &'static str { "Neigh" }
    fn play(self: Box<Self>, player: usize, history: &Actions) -> Option<Actions> {
        let latest_card = &history[0];
        if latest_card.card.as_any().is::<SuperNeigh>() {
            return None;
        }

        return Some(vec![
            Action {
                card: self,
                atype: ActionType::Instant,
                board: latest_card.board.clone()
            }, latest_card.clone()]);
    }

    fn as_any(&self) -> &dyn Any { self }
}


#[cfg(test)]
mod CardTest {
    use super::*;

    fn default_board() -> Board {
        return Board::new_base_game(2);
    }

    #[test]
    fn test_has_card() {
        let board = default_board();
        assert!(board.deck.has_card::<Neigh>(), "Should contain Neigh");
        assert!(board.deck.has_card::<SuperNeigh>(), "Should contain SuperNeigh");
    }

    fn test_count_card() {
        let board = default_board();
        assert!(board.deck.count_card::<Neigh>() == 3, "Should contain Neigh");
    }

    #[test]
    fn test_neigh_neigh() {
        let board = default_board();
        let neigh_action = Action {
            card: board.draw_specific_card::<Neigh>().unwrap().card,
            atype: ActionType::Instant,
            board: board.clone()
        };

        // Force a neigh on the neigh
        let forced_neigh = Box::new(Neigh {});
        let option = forced_neigh.play(0, &vec![neigh_action]).unwrap();

        assert!(option.len() == 2);
        assert!(option[0].card.as_any().is::<Neigh>());
        assert!(option[1].card.as_any().is::<Neigh>());
    }

    #[test]
    fn test_neigh_super_neigh() {
        let board = default_board();
        let neigh_action = Action {
            card: board.draw_specific_card::<SuperNeigh>().unwrap().card,
            atype: ActionType::Instant,
            board: board.clone()
        };

        // Force a neigh on the neigh
        let forced_neigh = Box::new(Neigh {});
        let option = forced_neigh.play(0, &vec![neigh_action]);
        assert!(option.is_none(), "Cannot neigh a super neigh.");
    }

    #[test]
    fn test_is_unicorn() {
        assert!(CardType::BasicUnicorn.is_unicorn() == true);
        assert!(CardType::MagicUnicorn.is_unicorn() == true);
        assert!(CardType::BabyUnicorn.is_unicorn() == true);
        assert!(CardType::Magic.is_unicorn() == false);
        assert!(CardType::Instant.is_unicorn() == false);
        assert!(CardType::Downgrade.is_unicorn() == false);
        assert!(CardType::Upgrade.is_unicorn() == false);
    }
}