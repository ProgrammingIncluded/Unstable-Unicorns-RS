use crate::state::*;

use std::rc::{Rc, Weak};
use std::cell::RefCell;

type Link<T> = Rc<RefCell<T>>;
type WeakLink<T> = Weak<RefCell<T>>;

#[derive(Debug)]
pub struct Node {
    children: Vec<Link<Node>>,
    parent: Option<WeakLink<Node>>,
    board: Board,
}

impl Node {
    fn new(children: Vec<Link<Node>>,
           parent: Option<WeakLink<Node>>,
           board: Board) -> Link<Self> {

        return Rc::new(RefCell::new(
            Node {
                children: children,
                parent: parent,
                board: board,
            }
        ));
    }
}

#[derive(Debug)]
struct GameTree {
    root: Link<Node>
}

impl GameTree {
    fn new(board: Board) -> GameTree {
        return GameTree {
            root: Node::new(
                vec![],
                None,
                board
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
    fn get_states(&self, tree: GameTree, player: u8, board: Board) {
        let ep = self.effect_phase(player, board);
        for e in ep {
        }
    }
    
    fn effect_phase(&self, player: u8, board: Board) -> Vec<Node> {
        return Vec::new();
    }
    
    fn draw_phase(&self, player: u8, board: Board) -> Vec<Node> {
        return Vec::new();
    }
    
    fn play_phase(&self, player: u8, board: Board) -> Vec<Node> {
        return Vec::new();
    }
}

mod GameTest {
    use super::*;
    use crate::cards::*;

    #[test]
    fn test_states() {
        let game = Game {};
        let tree = GameTree::new(Board::new_base_game(2));
        let board_start = tree.root.borrow().board.clone();
        let result = game.get_states(tree, 0, board_start);
    }

    #[test]
    fn test_tree() {
        let root = Node::new(
            vec![],
            None,
            Board::new_base_game(2)
        );

        {
            let mut root_value = root.borrow_mut();
    
            let leaf = Node::new(
                vec![],
                Some(Rc::downgrade(&root)),
                root_value.board.draw_specific_card::<Neigh>()
                    .unwrap().board
            );
            root_value.children = vec![leaf.clone()];
        }


        let tree = GameTree {
            root: root
        };

        let root_borrowed = tree.root.borrow();
        assert!(root_borrowed.children[0].borrow().board.deck.len() == root_borrowed.board.deck.len() - 1);
        assert!(Rc::ptr_eq(&root_borrowed.children[0].borrow().parent.as_ref().unwrap().upgrade().unwrap(), &tree.root));
    }
}