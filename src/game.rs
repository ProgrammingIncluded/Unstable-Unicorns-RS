use crate::state::*;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

type Link<T> = Rc<RefCell<T>>;
type WeakLink<T> = Weak<RefCell<T>>;

#[derive(Debug)]
pub struct Node {
    children: Vec<Link<Node>>,
    parent: Option<WeakLink<Node>>,
    action: Action,
}

impl Node {
    fn new(children: Vec<Link<Node>>,
           parent: Option<WeakLink<Node>>,
           action: Action) -> Link<Self> {

        return Rc::new(RefCell::new(
            Node {
                children: children,
                parent: parent,
                action: action,
            }
        ));
    }
}

#[derive(Debug)]
struct GameTree {
    root: Link<Node>
}

impl GameTree {
    fn new(action: Action) -> GameTree {
        assert!(action.atype == ActionType::GameStart);
        return GameTree {
            root: Node::new(
                vec![],
                None,
                action
            )
        };
    }
}

struct Game {}
impl Game {
    fn new() -> Self {
       return Game {};
    }

    // Generates and evalulates the game
    fn get_states(&self, player: usize, history: &History) {
        let ep = self.effect_phase(player, history);
    }
    
    fn effect_phase(&self, player: usize, history: &History) -> Link<Node> {
        // There must always be atleast one action
        assert!(history.len() >= 1, "There must always be an action to compute.");
        // There are really only two possible actions to take
        let latest_action = &history[history.len() - 1];
        match latest_action.atype {
            ActionType::ReactStart => {},
            ActionType::GameStart => {},
            _ => {assert!(false, "Must be an available action to trigger effect phase.")}
        }

        let new_action = Action::new_start(ActionType::ReactStart, latest_action.board.clone());
        let generate_children = |start: Action, history: History| -> Actions { 
                                        start.board
                                        .players[player]
                                        .stable
                                        .iter()
                                        .filter(|x| !start.atype.is_phase() || x.phase_playable().contains(&start.atype))
                                        .filter_map(|x| x.clone().react(player, &history))
                                        .collect()
                                    };

        let result_node = Node::new(vec![], None, new_action.clone());
        let mut stack: Vec<(Link<Node>, Action, History)> = vec![(
            result_node.clone(),
            new_action.clone(),
            history.clone()
        )];

        // Should we try to generate everything in a single phase? Yes.
        loop {
            match stack.pop() {
                Some((node, action, action_history)) => {
                    let mut new_actions: Vec<Action> = generate_children(action, action_history.clone());

                    let parent_rc = Some(Rc::downgrade(&node));
                    let mut new_nodes: Vec<Link<Node>> = new_actions.iter().map(|x| Node::new(vec![], parent_rc.clone(), x.clone())).collect();
                    node.borrow_mut().children = new_nodes.clone();

                    match (new_actions.pop(), new_nodes.pop()) {
                        (Some(a), Some(n)) => {
                            let mut new_history = action_history.clone();
                            new_history.push(Rc::new(a.clone()));
                            stack.push((n, a, new_history));
                        },
                        (None, None) => { continue; },
                        _ => { assert!(false, "Fatal, should have parity."); }
                    }
                },
                None => { break; }
            }
        }

        return result_node;
    }
    
    fn draw_phase(&self, player: usize, history: &History) -> Link<Node> {
        assert!(history.len() >= 1, "There must always be an action to compute.");
        let latest_action = &history[history.len() - 1];
        let phase_action = Action::new_start(ActionType::DrawStart, latest_action.board.clone());
        let phase_node = Node::new(vec![], None, phase_action.clone());

        let mut draw_nodes: Vec<Link<Node>> = Vec::new();
        for idx in 0..latest_action.board.deck.len() {
            let mut board_copy = latest_action.board.clone();
            let card = board_copy.deck.remove(idx);

            // Add current card to the machine
            board_copy.players[player].hand.push(card.clone());
            let new_action = Action {
                card: card,
                atype: ActionType::Draw,
                board: board_copy
            };

            let mut new_history = history.clone();
            new_history.push(Rc::new(new_action.clone()));
            let new_node = Node::new(vec![], Some(Rc::downgrade(&phase_node)), new_action);
            let react_node = self.react_phase(player, &new_history);
            new_node.borrow_mut().children = vec![react_node];
            draw_nodes.push(new_node);
        }

        phase_node.borrow_mut().children = draw_nodes;
        return phase_node;
    }

    fn react_phase(&self, player: usize, history: &History) -> Link<Node> {
        assert!(history.len() >= 1, "There must always be an action to compute.");
        // Create a react node
        let start_action = Action::new_start(ActionType::ReactStart, history[history.len() - 1].board.clone());
        let mut react_node = Node::new(vec![], None, start_action.clone());
        let new_node = self.effect_phase(player, &vec![Rc::new(start_action)]);
        react_node.borrow_mut().children = vec![new_node.clone()];
        new_node.borrow_mut().parent = Some(Rc::downgrade(&react_node));
        return react_node;
    }
    
    fn play_phase(&self, player: usize, history: &History) -> Link<Node> {
        assert!(history.len() >= 1, "There must always be an action to compute.");

        // Create a play node
        let phase_action = Action::new_start(ActionType::PlayStart, history[history.len() - 1].board.clone());
        let play_node = Node::new(vec![], None, phase_action.clone());

        let mut play_nodes: Vec<Link<Node>> = Vec::new();
        for idx in 0..phase_action.board.players[player].hand.len() {
            let mut board_copy = phase_action.board.clone();
            let card = board_copy.players[player].hand.remove(idx);

            board_copy.players[player].stable.push(card.clone());
            let new_action = Action {
                card: card,
                atype: ActionType::Place,
                board: board_copy
            };

            let mut new_history = history.clone();
            new_history.push(Rc::new(phase_action.clone()));
            let new_node = Node::new(vec![], Some(Rc::downgrade(&play_node)), new_action);
            let react_node = self.react_phase(player, &new_history);
            new_node.borrow_mut().children = vec![react_node];
            play_nodes.push(new_node);
        }

        play_node.borrow_mut().children = play_nodes;
        return play_node;
    }
}

mod GameTest {
    use super::*;
    use crate::cards::*;

    #[test]
    fn test_draw_phase() {
        let game = Game {};
        let board = Board::new_base_game(2);
        let deck_count = board.deck.len();
        let tree = GameTree::new(Action{
            card: Box::new(Neigh {}),
            atype: ActionType::GameStart,
            board: board
        });
        let history = vec![Rc::new(tree.root.borrow().action.clone())];
        let result = game.draw_phase(0, &history);

        for n in &result.borrow().children {
            let n = n.borrow();
            assert!(deck_count - n.action.board.deck.len() == 1, "Should have only drawn one card.");
            assert!(n.action.board.players[0].hand.len() == 1, "Should have one card in hand.");
        }
    }

    #[test]
    fn test_effect_phase() {
        let game = Game {};
        let tree = GameTree::new(Action{
            card: Box::new(Neigh {}),
            atype: ActionType::GameStart,
            board: Board::new_base_game(2)
        });
        let history = vec![Rc::new(tree.root.borrow().action.clone())];
        let result = game.effect_phase(0, &history);
    }

    #[test]
    fn test_tree() {
        let root = Node::new(
            vec![],
            None,
            Action{
                card: Box::new(Neigh {}),
                atype: ActionType::GameStart,
                board: Board::new_base_game(2)
            }
        );

        {
            let mut root_value = root.borrow_mut();
            let leaf = Node::new(
                vec![],
                Some(Rc::downgrade(&root)),
                root_value.action.board.draw_specific_card::<Neigh>().unwrap()
            );
            root_value.children = vec![leaf.clone()];
        }


        let tree = GameTree {
            root: root
        };

        let root_borrowed = tree.root.borrow();
        // Ugly but hopefully never have to do this in real code...
        // For the sake of a parent reference. Which may not be necessary... See if it is worth keeping.
        assert!(root_borrowed.children[0].borrow().action.board.deck.len() == root_borrowed.action.board.deck.len() - 1);
        assert!(Rc::ptr_eq(&root_borrowed.children[0].borrow().parent.as_ref().unwrap().upgrade().unwrap(), &tree.root));
    }
}