use crate::state::*;
use crate::cards::{BabyUnicorn, Card, QueryCards};

use std::rc::Rc;
use petgraph::visit::NodeIndexable;
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaChaRng;

use petgraph::graph::NodeIndex;
use petgraph::Graph;

#[derive(Debug)]
pub struct ActionEdge {
    pub card: Box<dyn Card>,
    pub atype: ActionType
}

impl From<&Action> for ActionEdge {
    fn from(value: &Action) -> Self {
        return ActionEdge {
            card: value.card.clone(),
            atype: value.atype.clone()
        }
    }
}

type GameGraph = Graph::<GameState, ActionEdge>;

impl GameState {
    fn new(board: &Board, phase: &PhaseType) -> Self {
        return GameState { board: board.clone(), phase: phase.clone() };
    }
}

struct Game { graph: GameGraph }
impl Game {
    fn new(board: &Board, setup: bool, seed: Option<[u8; 32]>) -> Self {
        let mut start_graph = GameGraph::new();

        let mut new_board = board.clone();
        if let Some(s) = seed {
            let mut rng = ChaChaRng::from_seed(s);
            new_board.deck.shuffle(&mut rng);
            new_board.nursery.shuffle(&mut rng);
        }

        // Do some extra setup to reduce tree depth that every game must do.
        // This part does not require any user input so can be done before tree is generated.
        if setup {
            let mut new_nursery = new_board.nursery;
            for idx in 0..new_board.players.len() {
                let (baby, new_nursery) = new_nursery.remove_one_card_with_type::<BabyUnicorn>().unwrap();
                new_board.players[idx].stable.push(baby);
            }
            new_board.nursery = new_nursery;

            // discard two for discard pile
            let discard_one = new_board.deck.pop().unwrap();
            let discard_two = new_board.deck.pop().unwrap();
            new_board.discard.push(discard_one);
            new_board.discard.push(discard_two);
        }

        start_graph.add_node(GameState::new(&new_board, &PhaseType::GameStart));
        return Game {graph: start_graph};
    }

    fn draw_phase(&mut self, player: usize, idx: &NodeIndex) {
        let cur_state: &GameState = self.graph.node_weight(*idx).unwrap();
        let mut board_copy = cur_state.board.clone();
        let card = board_copy.deck.remove(0);

        // Add current card to the machine
        board_copy.players[player].hand.push(card.clone());
        let new_node = GameState::new(&board_copy, &PhaseType::Draw);
        let new_action = Action {
            card: card,
            atype: ActionType::Draw,
            board: board_copy
        };

        let b_idx = self.graph.add_node(new_node);
        self.graph.add_edge(*idx, b_idx, ActionEdge::from(&new_action));
    }

    fn play_phase(&mut self, player: usize, idx: &NodeIndex) {
        let mut board_copy = self.graph.node_weight(*idx).unwrap().board.clone();
        for (h_idx, card) in board_copy.players[player].hand.iter().enumerate() {
            if !card.phase_playable().contains(&PhaseType::Play) {
                continue
            }

            // Check if its possible to play said card.
            // For this case, we don't care about history because its the first play of the stack.
            let mut board_copy = board_copy.clone();
            let card = board_copy.players[player].hand.remove(h_idx);

            // Location can change, so we play the card to resolve the action.
            let action = card.play(player, &self.graph.node_weight(*idx).unwrap(),&vec![]);
            if let None = action {
                continue;
            }
            let action = action.unwrap();
            let phase_node = GameState::new(&action.board, &PhaseType::Play);
            let b_idx = self.graph.add_node(phase_node);
            self.graph.add_edge(*idx, b_idx, ActionEdge::from(&action));
        }
    }
}

mod GameTest {

    use super::*;
    use crate::cards::*;

    #[test]
    fn test_play_phase() {
        // We only test the code in the phase and not per-card logic.
        let mut board = Board::new_base_game(2);

        // Put a neigh in hand to allow for playing in calculation.
        let (card, new_deck) = board.deck.remove_one_card_with_type::<BasicUnicorn>().unwrap();
        board.deck = new_deck;
        board.players[0].hand.push(card);

        let mut game = Game::new(&board, true, None);

        // We play which should have one neigh card to play.
        game.play_phase(0, &NodeIndex::new(0));

        // Two nodes, one start node, and one neigh play node.
        assert!(game.graph.node_count() >= 2);
        let gs: &GameState = game.graph.node_weight(NodeIndex::from(1)).unwrap();
        assert!(gs.board.discard.len() == 2);
        assert!(gs.board.players[0].stable.count_card::<BasicUnicorn>() == 1);
        assert!(gs.board.players[0].stable.count_card::<BabyUnicorn>() == 1);
    }

    #[test]
    fn test_draw_phase() {
        let board = Board::new_base_game(2);
        let mut game = Game::new(&board, false, None);
        let deck_count = board.deck.len();

        game.draw_phase(0, &NodeIndex::new(0));
        assert!(game.graph.node_count() >= 2);

        for out_going in &game.graph.raw_nodes()[1..] {
            assert!(deck_count - out_going.weight.board.deck.len() == 1, "Should have only drawn one card.");
            assert!(out_going.weight.board.players[0].hand.len() == 1, "Should have one card in hand.");
        }

        for edge in game.graph.raw_edges() {
            assert!(edge.weight.atype == ActionType::Draw, "Should be draw.");
        }
    }
}
