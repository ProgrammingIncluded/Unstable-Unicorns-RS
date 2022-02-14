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
    fn get_states(&self, history: &History, player: usize) {
        let ep = self.effect_phase(player, history);
        for e in ep {
            println!("{:?}", e);
        }
    }
    
    fn effect_phase(&self, player: usize, history: &History) -> Vec<Link<Node>> {
        // There must always be atleast one action
        assert!(history.len() >= 1, "There must always be an action to compute.");
        // There are really only two possible actions to take
        let latest_action = &history[history.len() - 1];
        match latest_action.atype {
            ActionType::React => {},
            ActionType::GameStart => {},
            ActionType::EffectStart => {},
            _ => {assert!(false, "Must be an available action to trigger effect phase.")}
        }

        // Generate initial children's react
        let generate_children = |start: &Action, history: &History| -> Vec<Action> { 
            start.board
                 .players[player]
                 .stable
                 .iter()
                 .filter_map(|x| x.clone().react(player, history))
                 .collect() 
        };

        // Pair of the node and the path in the tree to node
        // to represent the actions
        let mut stack: Vec<(Link<Node>, Action, History)> = Vec::new();
        {
            let start_actions: Vec<Action> = generate_children(&latest_action.clone(),&history);
            let start_nodes: Vec<Link<Node>> = start_actions.iter()
                                                            .map(|x| Node::new(vec![], None, x.clone()))
                                                            .collect();
            for i in 0..start_nodes.len() {
                stack.push((start_nodes[i].clone(), start_actions[i].clone(), history.clone()));
            }
        }

        // Should we try to generate everything?
        // Yes for now
        loop {
            match stack.pop() {
                Some((node, action, action_history)) => {
                },
                None => { break; }
            }
        }

        println!("{:?}", stack);
        return vec![];
    }
    
    fn draw_phase(&self, player: usize, board: Board) -> Vec<Link<Node>> {
        return Vec::new();
    }
    
    fn play_phase(&self, player: usize, board: Board) -> Vec<Link<Node>> {
        return Vec::new();
    }
}

mod GameTest {
    use super::*;
    use crate::cards::*;

    #[test]
    fn test_states() {
        let game = Game {};
        let tree = GameTree::new(Action{
            card: Box::new(Neigh {}),
            atype: ActionType::GameStart,
            board: Board::new_base_game(2)
        });
        let history = vec![Rc::new(tree.root.borrow().action.clone())];
        let result = game.get_states(&history, 0);
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
                root_value.action.board.draw_specific_card::<Neigh>()
                    .unwrap()
            );
            root_value.children = vec![leaf.clone()];
        }


        let tree = GameTree {
            root: root
        };

        let root_borrowed = tree.root.borrow();
        assert!(root_borrowed.children[0].borrow().action.board.deck.len() == root_borrowed.action.board.deck.len() - 1);
        assert!(Rc::ptr_eq(&root_borrowed.children[0].borrow().parent.as_ref().unwrap().upgrade().unwrap(), &tree.root));
    }
}