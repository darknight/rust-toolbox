/// BTree implement
/// Refer to:
/// https://www.geeksforgeeks.org/b-tree-set-1-introduction-2/
/// https://www.geeksforgeeks.org/b-tree-set-1-insert-2/
/// https://www.geeksforgeeks.org/b-tree-set-3delete/
///

use std::hash::Hash;
use std::fmt::Debug;
use std::collections::HashMap;

struct TreeNode<K: Eq + Hash + Ord + Copy, V> {
    is_leaf: bool,
    degree: usize,
    keys: Vec<K>,
    payload: HashMap<K, V>,
    children: Vec<Box<TreeNode<K, V>>>,
}

pub struct BTree<K: Eq + Hash + Ord + Copy, V> {
    root: Option<Box<TreeNode<K, V>>>,
    degree: usize,
}

impl<K, V> TreeNode<K, V> where K: Eq + Hash + Debug + Ord + Copy {

    pub fn new(is_leaf: bool, degree: usize) -> TreeNode<K, V> {
        TreeNode {
            is_leaf,
            degree: degree,
            keys: vec![],
            payload: HashMap::new(),
            children: vec![]
        }
    }

    pub fn is_full_node(&self) -> bool {
        self.keys.len() >= self.capacity()
    }

    pub fn traverse(&self, level: u8) {
        if self.is_leaf {
            print!("level-{} =>", level);
            for key in self.keys.iter() {
                print!("[{:?}]", key);
            }
            println!("");
        } else {
            for (i, key) in self.keys.iter().enumerate() {
                let child = self.children[i].as_ref();
                child.traverse(level + 1);
                println!("level-{} => [{:?}]", level, key);
            }
            let right_most_child = self.children.last().unwrap().as_ref();
            right_most_child.traverse(level + 1);
        }
    }

    pub fn search(&self, target: &K) -> Option<&V> {
        if self.payload.contains_key(target) {
            return self.payload.get(target);
        }

        if self.is_leaf {
            return None;
        }

        let index = self.keys.binary_search(target).unwrap_err();
        return self.children[index].as_ref().search(target);
    }

    fn insert_non_full(&mut self, key: K, value: V) {
        let mut index = self.keys.binary_search(&key).unwrap_err();
        if self.is_leaf {
            // There's no children for leaf node
            self.keys.insert(index, key);
            self.payload.insert(key, value);
        } else {
            if self.children[index].as_ref().is_full_node() {
                self.split_child(index);
                if self.keys[index] < key {
                    index += 1;
                }
            }
            self.children[index].as_mut().insert_non_full(key, value);
        }
    }

    fn split_child(&mut self, child: usize) {
        let move_up_key;
        let move_up_val;
        let mut node_z;
        {
            let node_y = self.children[child].as_mut();
            node_z = Self::new(node_y.is_leaf, node_y.degree);

            for moved_key in node_y.keys.drain(self.degree..) {
                let moved_val = node_y.payload.remove(&moved_key).unwrap();
                node_z.keys.push(moved_key);
                node_z.payload.insert(moved_key, moved_val);
            }

            if !node_y.is_leaf {
                let moved_children = node_y.children.drain(self.degree..);
                node_z.children.extend(moved_children);
            }

            move_up_key = node_y.keys.pop().unwrap();
            move_up_val = node_y.payload.remove(&move_up_key).unwrap();
        }
        self.payload.insert(move_up_key, move_up_val);
        self.keys.insert(child, move_up_key);
        self.children.insert(child + 1, Box::new(node_z));
    }

    fn capacity(&self) -> usize {
        self.degree * 2 - 1
    }

    fn meet_degree(&self) -> bool { self.keys.len() as usize >= self.degree }

    fn find_predecessor_of(&mut self, index: usize) -> K {
        let mut child = self.children[index].as_ref();
        while !child.is_leaf {
            child = child.children.last().unwrap();
        }

        *child.keys.last().unwrap()
    }

    fn find_successor_of(&mut self, index: usize) -> K {
        let mut child = self.children[index + 1].as_ref();
        while !child.is_leaf {
            child = child.children.first().unwrap();
        }

        *child.keys.first().unwrap()
    }

    fn merge_node(&mut self, mut node: Box<TreeNode<K, V>>) {

        self.keys.extend(node.keys.drain(..));
        self.payload.extend(node.payload.drain());
        self.children.extend(node.children.drain(..));
    }

    fn borrow_from_next_silbing<'a>(&mut self, key: K, index: usize) -> Option<V> {
        //FIXME: cargo build can pass, but rustc fail
//        let next_sibling = self.children[index + 1].as_mut();
//        let move_up_key = next_sibling.keys.remove(0);
//        let move_up_val = next_sibling.payload.remove(&move_up_key).unwrap();
//        let move_down_child = if next_sibling.is_leaf { None } else { Some(next_sibling.children.remove(0)) };

        let move_up_key;
        let move_up_val;
        let move_down_child;
        {
            let next_sibling = self.children[index + 1].as_mut();

            move_up_key = next_sibling.keys.remove(0);
            move_up_val = next_sibling.payload.remove(&move_up_key).unwrap();
            move_down_child = if next_sibling.is_leaf { None } else { Some(next_sibling.children.remove(0)) };
        }

        let move_down_key = self.keys.remove(index);
        let move_down_val = self.payload.remove(&move_down_key).unwrap();

        // put move up value to current node
        self.keys.insert(index, move_up_key);
        self.payload.insert(move_up_key, move_up_val);

        let goto_child = self.children[index].as_mut();
        // put move down value to goto child
        goto_child.keys.push(move_down_key);
        goto_child.payload.insert(move_down_key, move_down_val);
        move_down_child.map(|child| goto_child.children.push(child));

        return goto_child.delete_node(key);
    }

    fn borrow_from_prev_silbing(&mut self, key: K, index: usize) -> Option<V> {
        let move_up_key;
        let move_up_val;
        let move_down_child;
        {
            let prev_sibling = self.children[index - 1].as_mut();
            move_up_key = prev_sibling.keys.pop().unwrap();
            move_up_val = prev_sibling.payload.remove(&move_up_key).unwrap();
            move_down_child = if prev_sibling.is_leaf { None } else { prev_sibling.children.pop() };
        }

        let move_down_key = self.keys.remove(index - 1);
        let move_down_val = self.payload.remove(&move_down_key).unwrap();

        // put move up value to current node
        self.keys.insert(index - 1, move_up_key);
        self.payload.insert(move_up_key, move_up_val);

        let goto_child = self.children[index].as_mut();
        // put move down value to goto child
        goto_child.keys.insert(0, move_down_key);
        goto_child.payload.insert(move_down_key, move_down_val);
        move_down_child.map(|child| goto_child.children.push(child));

        return goto_child.delete_node(key);
    }

    fn merge_right_to_left(&mut self, key: K, index: usize) -> Option<V> {
        let move_down_key = self.keys.remove(index);
        let move_down_val = self.payload.remove(&move_down_key).unwrap();

        let right_child = self.children.remove(index + 1);
        let left_child = self.children[index].as_mut();

        left_child.keys.push(move_down_key);
        left_child.payload.insert(move_down_key, move_down_val);

        left_child.merge_node(right_child);

        return left_child.delete_node(key);
    }

    fn delete_node(&mut self, key: K) -> Option<V> {
        if self.is_leaf {
            // for leaf node, delete data directly
            match self.keys.binary_search(&key) {
                Ok(index) => {
                    self.keys.remove(index);
                    return self.payload.remove(&key);
                },
                Err(_) => return None,
            }
        } else {
            match self.keys.binary_search(&key) {
                Ok(index) => {
                    // for internal node, check the children key amount
                    let left_child_meet_degree = self.children[index].meet_degree();
                    let right_child_meet_degree = self.children[index + 1].meet_degree();
                    if left_child_meet_degree {
                        let pred_key = self.find_predecessor_of(index);

                        let left_child = self.children[index].as_mut();
                        let pred_val = left_child.delete_node(pred_key).unwrap();
                        self.keys[index] = pred_key;
                        self.payload.insert(pred_key, pred_val);

                        return self.payload.remove(&key);
                    } else if right_child_meet_degree {
                        let succ_key = self.find_successor_of(index);

                        let right_child = self.children[index + 1].as_mut();
                        let succ_val = right_child.delete_node(succ_key).unwrap();
                        self.keys[index] = succ_key;
                        self.payload.insert(succ_key, succ_val);

                        return self.payload.remove(&key);
                    } else {
                        return self.merge_right_to_left(key, index);
                    }
                },
                Err(index) => {
                    if self.children[index].as_ref().meet_degree() {
                        return self.children[index].as_mut().delete_node(key);
                    } else {
                        if index == 0 {
                            if self.children[1].as_ref().meet_degree() {
                                return self.borrow_from_next_silbing(key, index);
                            } else {
                                return self.merge_right_to_left(key, index);
                            }
                        }
                        else if index == self.keys.len() {
                            let prev_sibling_meet_degree = self.children[self.keys.len() - 1].meet_degree();
                            if prev_sibling_meet_degree {
                                return self.borrow_from_prev_silbing(key, index);
                            } else {
                                // index is the right most node, so -1 means left child
                                return self.merge_right_to_left(key, index - 1);
                            }
                        } else {
                            // FIXME: cargo build can pass, but rustc fail
//                            let prev_sibling = self.children[index - 1].as_ref();
//                            let next_sibling = self.children[index + 1].as_ref();
                            let prev_sibling_meet_degree = self.children[index - 1].meet_degree();
                            let next_sibling_meet_degree = self.children[index + 1].meet_degree();
                            if prev_sibling_meet_degree {
                                return self.borrow_from_prev_silbing(key, index);
                            } else if next_sibling_meet_degree {
                                return self.borrow_from_next_silbing(key, index);
                            } else {
                                // merge siblings
                                return self.merge_right_to_left(key, index);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<K, V> BTree<K, V> where K: Eq + Hash + Ord + Debug + Copy {

    pub fn new(degree: usize) -> BTree<K, V> {
        let root = TreeNode::new(true, degree);
        BTree {
            root: Some(Box::new(root)),
            degree
        }
    }

    pub fn traverse(&self) {
        self.root.as_ref().unwrap().traverse(0);
    }

    pub fn search(&self, target: &K) -> Option<&V> {
        self.root.as_ref().unwrap().search(target)
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.search(&key).is_some() {
            println!("Error: duplicate key [{:?}] to insert", key);
            return;
        }
        if self.root.as_ref().unwrap().is_full_node() {
            let mut new_root = TreeNode::new(false, self.degree);
            let old_root = self.root.take().unwrap();

            new_root.children.insert(0, old_root);
            new_root.split_child(0);

            let mut index = 0;
            if new_root.keys[0] < key {
                index = 1;
            }

            new_root.children[index].as_mut().insert_non_full(key, value);
            self.root = Some(Box::new(new_root));
        } else {
            self.root.as_mut().unwrap().insert_non_full(key, value);
        }
    }

    pub fn delete(&mut self, key: K) -> Option<V> {
        let result = self.root.as_mut().unwrap().delete_node(key);
        if self.root.as_ref().unwrap().keys.is_empty() {
            // shrink this tree
            let mut empty_root = self.root.take().unwrap();
            let new_root = empty_root.children.pop().unwrap();
            self.root = Some(new_root);
        }
        result
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn init_btree() -> BTree<u32, u32> {
        let mut btree: BTree<u32, u32> = BTree::new(3);
        btree.insert(1, 1);
        btree.insert(3, 3);
        btree.insert(7, 7);
        btree.insert(10, 10);
        btree.insert(11, 11);

        btree.insert(13, 13);
        btree.insert(14, 14);
        btree.insert(15, 15);

        btree.insert(18, 18);
        btree.insert(16, 16);
        btree.insert(19, 19);

        btree.insert(24, 24);
        btree.insert(25, 25);
        btree.insert(26, 26);

        btree.insert(21, 21);
        btree.insert(4, 4);
        btree.insert(5, 5);
        btree.insert(20, 20);
        btree.insert(22, 22);
        btree.insert(2, 2);

        btree.insert(17, 17);
        btree.insert(12, 12);

        btree.insert(6, 6);

        println!("After initialization ===============>");
        btree.traverse();

        btree
    }

    #[test]
    fn test_insert() {
        let mut btree: BTree<u32, &'static str> = BTree::new(3);
        btree.insert(10, "10");
        btree.insert(20, "20");
        btree.insert(5, "5");
        btree.insert(6, "6");
        btree.insert(12, "12");
        btree.insert(30, "30");
        btree.insert(7, "7");
        btree.insert(17, "17");

        btree.traverse();

        assert_eq!(btree.search(&6), Some(&"6"));
        assert_eq!(btree.search(&15), None);
    }

    #[test]
    fn test_delete1() {
        let mut btree = init_btree();
        // 14 will borrow from prev
        for k in [6, 13, 7, 4, 14].iter() {
            println!("Start deleting {} =================>", k);
            btree.delete(*k);
            btree.traverse();
            println!("Finish deleting {} =================>", k);
        }
    }

    #[test]
    fn test_delete2() {
        let mut btree = init_btree();
        // 6 will borrow from next
        for k in [5, 6].iter() {
            println!("Start deleting {} =================>", k);
            btree.delete(*k);
            btree.traverse();
            println!("Finish deleting {} =================>", k);
        }
    }

    #[test]
    fn test_delete3() {
        let mut btree = init_btree();

        for k in [6, 13, 7, 4, 2, 16].iter() {
            println!("Start deleting {} =================>", k);
            btree.delete(*k);
            btree.traverse();
            println!("Finish deleting {} =================>", k);
        }
    }
}

fn main() {}