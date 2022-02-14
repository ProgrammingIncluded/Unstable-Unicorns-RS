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

        let new_action = Action {
            card: latest_action.card.clone(),
            atype: latest_action.atype.clone(),
            board: latest_action.board.clone()
        };
        let mut stack: Vec<(Link<Node>, Action, History)> = vec![
            (Node::new(vec![], None, new_action.clone()),
             new_action.clone(),
             history.clone())
        ];

        // Should we try to generate everything in a single phase? Yes.
        loop {
            match stack.pop() {
                Some((node, action, action_history)) => {
                    let mut new_actions: Vec<Action> = Vec::new();
                    for card in &action.board.players[player].stable {
                        if !card.action_playable().contains(&action.atype) {continue;}
                        match card.clone().react(player, history) {
                            Some(v) => {new_actions.push(v);}
                            _ => {continue;}
                        }
                    }

                    let parent_rc = Some(Rc::downgrade(&node));
                    let new_nodes = new_actions.iter().map(|x| Node::new(vec![], parent_rc.clone(), x.clone())).collect();
                    node.borrow_mut().children = new_nodes;
                },
                None => { break; }
            }
        }

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