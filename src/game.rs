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
    fn get_states(&self, tree: GameTree, player: usize, board: Board) {
        let ep = self.effect_phase(player, board);
        for e in ep {
            println!("{:?}", e);
        }
    }
    
    fn effect_phase(&self, player: usize, board: Board) -> Vec<Link<Node>> {
        // let new_nodes = vec![];
        let generate_children = |b: &Board| -> Vec<Actions> { 
            b.players[player]
              .stable
              .iter()
              .filter_map(|x| x.react(player, &vec![]))
              .collect() 
        };

        let action_to_node = |a: &Actions| -> Vec<Link<Node>> {
            a.iter().map(|x| Node::new(vec![], None, x.clone())).collect()
        };

        // Pair of the node and the path in the tree to node
        // to represent the actions
        let mut stack: Vec<(Link<Node>, Actions)> = Vec::new();
        {
            let start_actions: Vec<Actions> = generate_children(&board);
            let start_nodes: Vec<Link<Node>> = start_actions.iter().map(|x| Node::new(vec![], None, x[0].clone())).collect();
            for i in 0..start_nodes.len() {
                stack.push((start_nodes[i].clone(), start_actions[i].clone()));
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
        let board_start = tree.root.borrow().action.board.clone();
        let result = game.get_states(tree, 0, board_start);
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