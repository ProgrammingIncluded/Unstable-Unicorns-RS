use crate::state::*;

use std::rc::Rc;
use std::cell::RefCell;

type Link<T> = Rc<RefCell<T>>;

#[derive(Debug)]
pub struct Node {
    children: Vec<Link<Node>>,
    parent: Option<Link<Node>>,
    board: Box<Board>
}

impl Node {
    fn new(children: Vec<Link<Node>>,
           parent: Option<Link<Node>>,
           board: Box<Board>) -> Link<Self> {
        return Rc::new(RefCell::new(
            Node {
                children: children,
                parent: parent,
                board: board
            }
        ));
    }
}

#[derive(Debug)]
pub struct GameTree {
    root: Link<Node>
}

// impl GameTree {
    // // Generates and evalulates the game
    // fn get_states(player: u8, n: Link<Node>) {
        // let ep = effect_phase(player, b);
        // for e in ep {
            // draw_phase(player, e);
        // }
    // }
    
    // fn effect_phase(player: u8, n: Link<Node>) -> Vec<Link<Node>> {

    // }
    
    // fn draw_phase(player: u8, n: Link<Node>) {
    
    // }
    
    // fn play_phase(player: u8, n: Link<Node>) -> Vec<Link<Node>> {
        
    // }
// }




mod GameTest {
    use super::*;
    use crate::cards::*;

    #[test]
    fn test_tree() {
        let mut root = Node::new(
            vec![],
            None,
            Box::new(Board::new_base_game(2))
        );

        let leaf = Node::new(
            Vec::new(),
            Some(root.clone()),
            Box::new(root.borrow().board.draw_specific_card::<Neigh>()
                 .unwrap().board)
        );

        root.borrow_mut().children = vec![leaf.clone()];

        let tree = GameTree {
            root: root
        };

        assert!(tree.root.borrow().children[0].borrow().board.deck.len() == 1);
    }
}