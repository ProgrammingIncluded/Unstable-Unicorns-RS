use crate::state::*;
use crate::cards::{BabyUnicorn, Card, QueryCards};

use std::rc::Rc;
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaChaRng;

use petgraph::graph::NodeIndex;
use petgraph::Graph;

#[derive(Debug)]
pub struct GameState {
    board: Board,
    phase: PhaseType
}

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

    fn draw_phase(&self, player: usize, cur_state: &GameState, idx: &NodeIndex) -> GameGraph  {
        let mut graph = GameGraph::new();
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

        let b_idx = graph.add_node(new_node);
        graph.add_edge(*idx, b_idx, ActionEdge::from(&new_action));

        return graph;
    }

    fn play_phase(&self, player: usize, cur_state: &GameState, idx: &NodeIndex) -> GameGraph {
        let mut graph = GameGraph::new();
        for (h_idx, card) in cur_state.board.players[player].hand.iter().enumerate() {
            if !card.phase_playable().contains(&PhaseType::Play) {
                continue
            }

            // Check if its possible to play said card.
            // For this case, we don't care about history because its the first play of the stack.
            let mut board_copy = cur_state.board.clone();
            let phase_node = GameState::new(&cur_state.board, &PhaseType::Play);
            let card = board_copy.players[player].hand.remove(h_idx);

            // Location can change, so we play the card to resolve the action.
            let action = card.play(player, &vec![]);
            if let None = action {
                continue;
            }

            let action = action.unwrap();
            let b_idx = graph.add_node(phase_node);
            graph.add_edge(*idx, b_idx, ActionEdge::from(&action));
        }

        return graph;
    }
}

mod GameTest {

    use super::*;
    use crate::cards::*;

    #[test]
    fn test_draw_phase() {
        let board = Board::new_base_game(2);
        let game = Game::new(&board, false, None);
        let deck_count = board.deck.len();

        let result = game.draw_phase(0, &game.graph.raw_nodes()[0].weight, &NodeIndex::new(0));
        assert!(result.node_count() >= 1);

        for out_going in &result.raw_nodes()[1..] {
            assert!(deck_count - out_going.weight.board.deck.len() == 1, "Should have only drawn one card.");
            assert!(out_going.weight.board.players[0].hand.len() == 1, "Should have one card in hand.");
        }

        for edge in result.raw_edges() {
            assert!(edge.weight.atype == ActionType::Draw, "Should be draw.");
        }
    }

    #[test]
    fn test_play_phase() {
        let board = Board::new_base_game(2);
        let game = Game::new(&board, false, None);
        let deck_count = board.deck.len();

        // TODO: Finalize  node return.
        let result = game.draw_phase(0, &game.graph.raw_nodes()[0].weight, &NodeIndex::new(0));
    }

}
