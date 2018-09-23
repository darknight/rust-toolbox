#[allow(unused_imports)]
use std::ptr;
use std::cmp;
use std::cell::{Ref, Cell, RefCell};
use std::rc::Rc;
use std::cell::RefMut;
use std::fmt::{Debug, Display, Formatter};
use std::fmt;
use std::collections::VecDeque;


pub struct AvlTree<T> {
    root: Option<Rc<RefCell<Box<AvlNode<T>>>>>,
}

pub struct AvlNode<T> {
    data: T,
    left: Option<Rc<RefCell<Box<AvlNode<T>>>>>,
    right: Option<Rc<RefCell<Box<AvlNode<T>>>>>,
    height: usize,
}

impl<T> AvlNode<T> where T: Ord {

    fn left_height(self: &Self) -> usize {
        match self.left {
            None => 0,
            Some(ref node) => node.borrow().height
        }
    }

    fn right_height(&self) -> usize {
        match self.right {
            None => 0,
            Some(ref node) => node.borrow().height
        }
    }

    fn single_rotate_with_left(root: Rc<RefCell<Box<AvlNode<T>>>>) -> Rc<RefCell<Box<AvlNode<T>>>> {
        let new_root = root.borrow_mut().left.take().unwrap();
        let new_left_of_root = new_root.borrow_mut().right.take();

        root.borrow_mut().left = new_left_of_root;

        let new_height = cmp::max(root.borrow().left_height(),
                                  root.borrow().right_height()) + 1;
        root.borrow_mut().height = new_height;

        new_root.borrow_mut().right = Some(root);
        let new_root_new_height = cmp::max(new_root.borrow().left_height(),
                                           new_root.borrow().right_height()) + 1;
        new_root.borrow_mut().height = new_root_new_height;

        new_root
    }

    fn single_rotate_with_right(root: Rc<RefCell<Box<AvlNode<T>>>>) -> Rc<RefCell<Box<AvlNode<T>>>> {

        let new_root = root.borrow_mut().right.take().unwrap();
        let new_right_of_root = new_root.borrow_mut().left.take();

        root.borrow_mut().right = new_right_of_root;
        let new_height = cmp::max(root.borrow().left_height(),
                                  root.borrow().right_height()) + 1;
        root.borrow_mut().height = new_height;

        new_root.borrow_mut().left = Some(root);
        let new_root_new_height = cmp::max(new_root.borrow().left_height(),
                                           new_root.borrow().right_height()) + 1;
        new_root.borrow_mut().height = new_root_new_height;

        new_root
    }

    fn double_rotate_with_left(root: Rc<RefCell<Box<AvlNode<T>>>>) -> Rc<RefCell<Box<AvlNode<T>>>> {
        let left = root.borrow_mut().left.take().unwrap();
        root.borrow_mut().left = Some(Self::single_rotate_with_right(left));
        Self::single_rotate_with_left(root)
    }

    fn double_rotate_with_right(root: Rc<RefCell<Box<AvlNode<T>>>>) -> Rc<RefCell<Box<AvlNode<T>>>> {
        let right = root.borrow_mut().right.take().unwrap();
        root.borrow_mut().right = Some(Self::single_rotate_with_left(right));
        Self::single_rotate_with_right(root)
    }

    pub fn insert_node(root: Rc<RefCell<Box<AvlNode<T>>>>, leaf: Rc<RefCell<Box<AvlNode<T>>>>)
        -> Rc<RefCell<Box<AvlNode<T>>>> {

        let new_root; // new root for each iteration
//        let current_root = root.borrow_mut();
        if root.borrow().data > leaf.borrow().data {
            let current_left = root.borrow_mut().left.take();
            if current_left.is_none() {
                root.borrow_mut().left = Some(Rc::clone(&leaf));
            } else {
                let raw_left = Rc::clone(&current_left.unwrap());
                let new_left = Self::insert_node(raw_left, Rc::clone(&leaf));
                root.borrow_mut().left = Some(new_left);
            }

            let new_height = cmp::max(root.borrow().left_height(), root.borrow().right_height()) + 1;
            root.borrow_mut().height = new_height;

            if root.borrow().left_height() - root.borrow().right_height() == 2 {
                let is_single_rotate = root.borrow().left.as_ref().map_or(false,|node| {
                    if leaf.borrow().data < node.borrow().data { true } else { false }
                });
                if is_single_rotate {
                    new_root = Self::single_rotate_with_left(Rc::clone(&root));
                } else {
                    new_root = Self::double_rotate_with_left(Rc::clone(&root));
                }
            } else {
                new_root = Rc::clone(&root);
            }
        } else if root.borrow().data < leaf.borrow().data {
            let current_right = root.borrow_mut().right.take();
            if current_right.is_none() {
                root.borrow_mut().right = Some(Rc::clone(&leaf));
            } else {
                let raw_right = Rc::clone(&current_right.unwrap());
                let new_right = Self::insert_node(raw_right, Rc::clone(&leaf));
                root.borrow_mut().right = Some(new_right);
            }

            let new_height = cmp::max(root.borrow().left_height(), root.borrow().right_height()) + 1;
            root.borrow_mut().height = new_height;

            if root.borrow().right_height() - root.borrow().left_height() == 2 {
                let is_single_rotate = root.borrow().right.as_ref().map_or(false,|node| {
                    if leaf.borrow().data > node.borrow().data { true } else { false }
                });
                if is_single_rotate {
                    new_root = Self::single_rotate_with_right(Rc::clone(&root));
                } else {
                    new_root = Self::double_rotate_with_right(Rc::clone(&root));
                }
            } else {
                new_root = Rc::clone(&root);
            }
        } else {
            new_root = root;
        }

        new_root
    }
}

//impl<T> Debug for AvlNode<T> where T: Debug {
//
//    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
//        self.left.fmt(f);
//        self.right.fmt(f);
//        write!(f, "{:?}", self.data)
//    }
//}

impl<T> AvlTree<T> {
    pub fn new() -> AvlTree<T> {
        AvlTree { root: None }
    }
}

impl<T> AvlTree<T> where T: Ord {

    pub fn insert(&mut self, element: T) {
        let node = AvlNode {
            data: element,
            left: None,
            right: None,
            height: 1
        };

        if self.root.is_none() {
            self.root = Some(Rc::new(RefCell::new(Box::new(node))));
            return;
        } else {
            let root = self.root.take().unwrap();
            let new_root = AvlNode::insert_node(
                root,
                Rc::new(RefCell::new(Box::new(node)))
            );
//            let rt = Rc::try_unwrap(new_root).ok().unwrap().into_inner();
            self.root = Some(new_root);
        }
    }

//    pub fn find(&self, element: &T) -> Option<Cell<AvlNode<T>>> {
//        unimplemented!()
//    }
//
//    pub fn find_min(&self) -> &T {
//        unimplemented!()
//    }
//
//    pub fn find_max(&self) -> &T {
//        unimplemented!()
//    }
//
//    pub fn delete(&mut self, element: &T) -> Option<Cell<AvlNode<T>>> {
//        unimplemented!()
//    }
}

impl<T> AvlTree<T> where T: Debug {

    pub fn pprint(&self) {

        if self.root.is_none() {
            println!("[]")
        }
        else {
            let mut queue: VecDeque<(Rc<RefCell<Box<AvlNode<T>>>>, char, Rc<RefCell<Box<AvlNode<T>>>>)> = VecDeque::new();
            if let Some(ref root) = self.root {
                println!("[{:?}, {:?}]", root.borrow().data, "*");
                if let Some(ref left) = root.borrow().left {
                    queue.push_back((Rc::clone(left), 'L', Rc::clone(root)));
                }
                if let Some(ref right) = root.borrow().right {
                    queue.push_back((Rc::clone(right), 'R',  Rc::clone(root)));
                }
            }

            while !queue.is_empty() {
                let (node, side, parent) = queue.pop_front().unwrap();
                let as_parent1 = Rc::clone(&node);
                let as_parent2 = Rc::clone(&node);
                let raw_node = node.borrow();
                println!("[{:?} - {} - {:?}]", raw_node.data, side, parent.borrow().data);
                {
                    if let Some(ref left) = raw_node.left {
                        queue.push_back((Rc::clone(left), 'L', as_parent1));
                    }
                    if let Some(ref right) = raw_node.right {
                        queue.push_back((Rc::clone(right), 'R', as_parent2));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::AvlTree;

    #[test]
    fn simple_singe_rotate() {
        let mut tree = AvlTree::new();
        tree.insert(3);
        tree.insert(2);
        tree.insert(1);

        tree.pprint();
        assert!(true);
    }

    #[test]
    fn simple_singe_rotate2() {
        let mut tree = AvlTree::new();
        tree.insert(3);
        tree.insert(2);
        tree.insert(1);

        for i in &[4,5,6,7] {
            tree.insert(*i);
        }

        tree.pprint();
        assert!(true);
    }

    #[test]
    fn simple_double_rotate() {
        let mut tree = AvlTree::new();
        tree.insert(3);
        tree.insert(2);
        tree.insert(1);

        for i in &[4,5,6,7] {
            tree.insert(*i);
        }

        tree.insert(16);
        tree.insert(15);

        tree.pprint();
        assert!(true);
    }

    #[test]
    fn simple_mixed_rotate() {
        let mut tree = AvlTree::new();
        tree.insert(3);
        tree.insert(2);
        tree.insert(1);

        for i in &[4,5,6,7] {
            tree.insert(*i);
        }

        // insert 16, 15 .. 10
        for i in (10..17).rev() {
            tree.insert(i);
        }

        tree.insert(8);
        tree.insert(9);

        tree.pprint();
        assert!(true);
    }
}
