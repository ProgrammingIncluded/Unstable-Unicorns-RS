// STD
use std::any::{Any, TypeId};
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

// EXT
use dyn_clone::DynClone;

// UU
use crate::state::{Action, ActionType, Board, GameState, History, PhaseType, ReactResult, ReactAction};

#[derive(Debug, Clone)]
enum CardType {
    Null,
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

    // Always assumes card has already been taken from the hand.
    fn play(self: Box<Self>, player: usize, cur_state: &GameState, history: &History) -> ReactResult { Ok(vec![]) }
    fn react(self: Box<Self>, player: usize, cur_state: &GameState, history: &History) -> ReactResult { Ok(vec![]) }

    fn effect(&self, player: usize, cur_state: &GameState, history: &History) -> ReactResult { Ok(vec![]) }
    fn destroy(&self, player: usize, cur_state: &GameState, history: &History) -> ReactResult { Ok(vec![]) }
    fn steal(&self, player: usize, cur_state: &GameState, history: &History) -> ReactResult { Ok(vec![]) }

    /// Determines if the current card can play in a start phase.
    fn phase_playable(&self) -> &'static [PhaseType] {
        match self.ctype() {
            CardType::Instant => {return &[PhaseType::Draw, PhaseType::Effect, PhaseType::React]}
            _ => {return &[PhaseType::Draw, PhaseType::Effect, PhaseType::Play, PhaseType::React]}
        }
    }

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
pub struct BasicUnicorn {}
impl Card for BasicUnicorn {
    fn ctype(&self) -> CardType { CardType::BasicUnicorn }
    fn name(&self) -> &'static str { "Basic Unicorn" }
    fn play(self: Box<Self>, player: usize, cur_state: &GameState, history: &History) -> ReactResult {
        let mut latest_board = cur_state.board.clone();
        latest_board.players[player].stable.push(self.clone());

        return Ok(vec![
            ReactAction::from(
                &Action {
                    card: self,
                    atype: ActionType::Place,
                    board: latest_board,
                }
            )
        ]);
    }

    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct UnicornPhoenix {}
impl Card for UnicornPhoenix {
    fn ctype(&self) -> CardType { CardType::MagicUnicorn }
    fn name(&self) -> &'static str { "Unicorn Phoenix" }
    fn play(self: Box<Self>, player: usize, cur_state: &GameState, history: &History) -> ReactResult {
        let mut latest_board = cur_state.board.clone();
        latest_board.players[player].stable.push(self.clone());

        return Ok(vec![
            ReactAction::from(
                &Action {
                    card: self,
                    atype: ActionType::Place,
                    board: latest_board,
                }
            )
        ]);
    }

    fn effect(&self, player: usize, cur_state: &GameState, history: &History) -> ReactResult {
        let last_action = history.last().unwrap();
        if !last_action.card.as_any().is::<Self>() {
            return Ok(vec![]);
        }
        // Check the type of effect.
        let mut final_result = vec![];

        let mut new_board = cur_state.board.clone();
        let own_card = new_board.discard.pop().unwrap();
        assert!(own_card.as_any().is::<Self>(), "Invalid discard stack detected.");
        new_board.players[player].stable.push(own_card);
        let effect_action = Action {
            card: last_action.card.clone(),
            atype: ActionType::Revive,
            board: new_board
        };

        if last_action.atype == ActionType::Destroy || last_action.atype == ActionType::Sacrifice {
            let hand = &cur_state.board.players[player].hand;
            hand.iter().enumerate().map(|(idx, h)| {
                let mut new_board = cur_state.board.clone();
                new_board.players[player].hand.remove(idx);
                new_board.discard.push(h.clone());


                let follow_up = Action {
                    card: h.clone(),
                    atype: ActionType::Discard,
                    board: new_board
                };

                final_result.push(ReactAction {
                    effect_action: effect_action.clone(),
                    follow_up: Some(follow_up),
                    response: vec![]
                });
            }).count();
        }

        return Ok(final_result);
    }

    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct BabyUnicorn {}
impl Card for BabyUnicorn {
    fn ctype(&self) -> CardType { CardType::BabyUnicorn }
    fn name(&self) -> &'static str { "Baby Unicorn" }
    fn play(self: Box<Self>, player: usize, cur_state: &GameState, history: &History) -> ReactResult {
        let mut latest_board = cur_state.board.clone();
        latest_board.players[player].stable.push(self.clone());

        return Ok(vec![
            ReactAction::from(
                &Action {
                    card: self,
                    atype: ActionType::Place,
                    board: latest_board,
                }
            )
        ]);
    }

    fn as_any(&self) -> &dyn Any { self }
}


#[derive(Debug, Clone)]
pub struct SuperNeigh {}
impl Card for SuperNeigh {
    fn ctype(&self) -> CardType { CardType::Instant }
    fn name(&self) -> &'static str { "Super Neigh" }
    fn play(self: Box<Self>, player: usize, cur_state: &GameState, history: &History) -> ReactResult {
        let mut latest_board = cur_state.board.clone();
        latest_board.discard.push(self.clone());

        return Ok(vec![
            ReactAction::from(
                &Action {
                    card: self,
                    atype: ActionType::Instant,
                    board: latest_board
                }
            )
        ]);
    }

    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct Neigh {}
impl Card for Neigh {
    fn ctype(&self) -> CardType { CardType::Instant }
    fn name(&self) -> &'static str { "Neigh" }
    fn react(self: Box<Self>, player: usize, cur_state: &GameState, history: &History) -> ReactResult {
        if history.len() < 1 {
            // Cannot play instant without a reaction.
            return Ok(vec![]);
        }

        let latest_action = &history.last().unwrap();
        if latest_action.card.as_any().is::<SuperNeigh>() {
            return Ok(vec![]);
        }

        let mut latest_board = latest_action.board.clone();
        latest_board.discard.push(self.clone());

        return Ok(vec![
            ReactAction::from(
                &Action {
                    card: self,
                    atype: ActionType::Instant,
                    board: latest_board
                }
            )
        ]);
    }

    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct UnicornPoison {}
impl Card for UnicornPoison {
    fn ctype(&self) -> CardType { CardType::Magic }
    fn name(&self) -> &'static str { "Unicorn Poison" }
    fn play(self: Box<Self>, player: usize, cur_state: &GameState, history: &History) -> ReactResult {
        let mut latest_board = cur_state.board.clone();
        latest_board.discard.push(self.clone());

        let mut result = vec![];

        let effect_action = Action {
            card: self,
            atype: ActionType::Discard,
            board: latest_board.clone(),
        };

        for (p_idx, p) in latest_board.players.iter().enumerate() {
            for (idx, k) in p.stable.iter().enumerate() {
                if !k.ctype().is_unicorn() {
                    continue
                }

                let mut follow_board = latest_board.clone();
                follow_board.players[p_idx].stable.remove(idx);
                follow_board.discard.push(k.clone());
                let follow_up = Action {
                    card: k.clone(),
                    atype: ActionType::Destroy,
                    board: follow_board
                };

                result.push(ReactAction {
                    follow_up: Some(follow_up),
                    effect_action: effect_action.clone(),
                    response: vec![]
                })
            }
        }

        return Ok(result);
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
            card: board.draw_specific_card::<Neigh>().unwrap().unwrap().card,
            atype: ActionType::Instant,
            board: board.clone()
        };

        let game_state = GameState {
            board,
            phase: PhaseType::Play,
            react_metadata: None
        };

        // Force a neigh on the neigh
        let forced_neigh = Box::new(Neigh {});
        let option = forced_neigh.react(0,
                                        &game_state,
                                        &vec![neigh_action]).unwrap();
        assert!(option.len() == 1);
        let option = &option[0];

        assert!(option.effect_action.card.as_any().is::<Neigh>());
        assert!(option.effect_action.board.discard.len() == 1);
        assert!(option.effect_action.board.discard.has_card::<Neigh>());
    }

    #[test]
    fn test_neigh_super_neigh() {
        let board = default_board();
        let neigh_action = Action {
            card: board.draw_specific_card::<SuperNeigh>().unwrap().unwrap().card,
            atype: ActionType::Instant,
            board: board.clone()
        };

        let game_state = GameState {
            board,
            phase: PhaseType::Play,
            react_metadata: None
        };

        // Force a neigh on the neigh
        let forced_neigh = Box::new(Neigh {});
        let option = forced_neigh.react(0, &game_state, &vec![neigh_action]);
        assert!(option.unwrap().len() == 0, "Cannot neigh a super neigh.");
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
