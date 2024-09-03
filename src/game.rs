use crate::state::*;
use crate::cards::{BabyUnicorn, Card, Cards, QueryCards};

use std::collections::HashMap;
use std::rc::Rc;
use petgraph::visit::NodeIndexable;
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaChaRng;

use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::visit::{EdgeRef, Bfs};
use petgraph::{Graph, Incoming};

#[derive(Clone, Debug)]
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

    fn draw_phase(&mut self, player: usize, idx: &NodeIndex) -> Result<(), LogicError>{
        let cur_state: &GameState = self.graph.node_weight(*idx).unwrap();
        let mut board_copy = cur_state.board.clone();
        let card = board_copy.deck.remove(0);

        // Add current card to the machine
        board_copy.players[player].hand.push(card.clone());
        let new_node = GameState::new(&board_copy, &PhaseType::React);
        let new_action = Action {
            card: card,
            atype: ActionType::Draw,
            board: board_copy
        };

        let b_idx = self.graph.add_node(new_node);
        self.graph.add_edge(*idx, b_idx, ActionEdge::from(&new_action));
        return Ok(());
    }

    fn effect_phase(&mut self, player: usize, a_idx: &EdgeIndex) -> Result<(), LogicError> {
        let node_idx = self.graph.edge_endpoints(*a_idx).unwrap().1;
        let edge_action = self.graph.edge_weight_mut(*a_idx).unwrap().clone();
        let game_state = self.graph.node_weight(node_idx).unwrap().clone();

        let mut _generate_actions = |container: &Cards| -> Result<(), LogicError> {
                for card in container {
                    let action = Action {
                        atype: edge_action.atype.clone(),
                        card: edge_action.card.clone(),
                        board: game_state.board.clone()
                    };
                    let reactions = card.effect(player, &game_state, &vec![action])?;

                    if reactions.len() <= 0 {
                        continue;
                    }

                    for reaction in reactions {
                        let action = &reaction.effect_action;
                        let mut phase_node = GameState::new(&action.board, &PhaseType::React);
                        phase_node.react_metadata = Option::<ReactMetadata>::from(&reaction);
                        let b_idx = self.graph.add_node(phase_node);
                        self.graph.add_edge(node_idx, b_idx, ActionEdge::from(action));
                    }
                }

                return Ok(());
            };

        // Any card can have an effect, so it is important that we keep track of the previous Node's phase
        // and use that as history.
        for hand_idx in 0..game_state.board.players.len() {
            _generate_actions(&game_state.board.players[hand_idx].hand)?;
            _generate_actions(&game_state.board.players[hand_idx].stable)?;
        }

        // We also need to add an no-op option.
        let mut phase_node = GameState::new(&game_state.board, &PhaseType::React);
        let no_idx = self.graph.add_node(phase_node);
        self.graph.add_edge(node_idx, no_idx, ActionEdge { card: edge_action.card.clone(), atype: ActionType::NoOp });

        // We can just resolve the react phase here recursively. :O Just need to check history.

        return Ok(());
    }

    fn play_phase(&mut self, player: usize, idx: &NodeIndex) -> Result<(), LogicError> {
        let board = self.graph.node_weight(*idx).unwrap().board.clone();
        for (h_idx, card) in board.players[player].hand.iter().cloned().enumerate() {
            if !card.phase_playable().contains(&PhaseType::Play) {
                continue
            }

            // Check if its possible to play said card.
            // For this case, we don't care about history because its the first play of the stack.
            let mut board_copy = board.clone();
            board_copy.players[player].hand.remove(h_idx);

            // Location can change, so we play the card to resolve the action.
            let actions = card.play(player, &self.graph.node_weight(*idx).unwrap(), &vec![])?;
            if actions.len() <= 0 {
                continue;
            }

            for action in actions {
                let mut phase_node = GameState::new(&action.effect_action.board, &PhaseType::React);
                if let Some(_) =  action.follow_up {
                    phase_node.react_metadata = Option::<ReactMetadata>::from(&action);
                }
                let b_idx = self.graph.add_node(phase_node);
                self.graph.add_edge(*idx, b_idx, ActionEdge::from(&action.effect_action));
            }
        }

        return Ok(());
    }
}

mod GameTest {

    use super::*;
    use crate::cards::*;

    #[test]
    fn test_effect_phase() {
        let mut board = Board::new_base_game(2);

        // We grab a unicorn phoenix and put it into the stable first.
        let (phoenix_card, new_deck) = board.deck.remove_one_card_with_type::<UnicornPhoenix>().unwrap();
        let (unicorn_poison, new_deck) = new_deck.remove_one_card_with_type::<UnicornPoison>().unwrap();
        let (neigh, new_deck) = new_deck.remove_one_card_with_type::<Neigh>().unwrap();

        board.deck = new_deck;
        board.players[0].stable.push(phoenix_card);
        board.players[0].hand.push(neigh);
        board.players[1].hand.push(unicorn_poison);

        // first we play the phoenix card
        let mut game = Game::new(&board, true, None);

        // Start with playing poison.
        game.play_phase(1, &NodeIndex::new(0)).unwrap();
        assert!(game.graph.node_count() == 4);

        let mut save_edge = None;
        let mut save_node = None;
        let mut bfs = Bfs::new(&game.graph, NodeIndex::new(0));
        let mut skip_first = true;
        while let Some(nx) = bfs.next(&game.graph) {
            if skip_first {
                skip_first = false;
                continue;
            }

            let weight = game.graph.node_weight(nx).unwrap();
            assert!(weight.phase == PhaseType::React);
            let follow_up = &weight.react_metadata.as_ref().unwrap().follow_up;
            // Find the card that we care about.
            if follow_up == &ResponseOp::Destroy {
                let mut edges = game.graph.neighbors_directed(nx, Incoming).detach();
                while let Some(edge) = edges.next_edge(&game.graph) {
                    save_node = Some(nx);
                    save_edge = Some(edge);
                }
            }
        }

        assert!(save_edge.is_some());
        let save_edge = save_edge.unwrap();
        let save_node = save_node.unwrap();

        // Try to destroy our unicorn card.
        game.effect_phase(0, &save_edge).unwrap();

        // Verify the generated payload.
        let edge_count: Vec<_> = game.graph.edges(save_node).collect();
        assert!(edge_count.len() == 1);
    }

    #[test]
    fn test_play_phase() {
        // We only test the code in the phase and not per-card logic.
        let mut board = Board::new_base_game(2);

        // Put a basic unicorn in hand to allow for playing in calculation.
        let (card, new_deck) = board.deck.remove_one_card_with_type::<BasicUnicorn>().unwrap();
        board.deck = new_deck;
        board.players[0].hand.push(card);

        let mut game = Game::new(&board, true, None);

        // We play which should have one neigh card to play.
        game.play_phase(0, &NodeIndex::new(0)).unwrap();

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

        game.draw_phase(0, &NodeIndex::new(0)).unwrap();
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
