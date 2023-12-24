use std::ops::{Index, IndexMut};

use crate::arena::{Arena, Id};

#[derive(Debug)]
pub struct TrieNode {
    // parent: usize,
    // is_end: bool,
    left: Id,  // "false"-child
    right: Id, // "true"-child
}

impl TrieNode {
    pub fn new(_parent: Id) -> Self {
        Self {
            // parent,
            left: 0,
            right: 0,
            // is_end: false,
        }
    }
}

#[derive(Debug)]
pub struct Trie {
    nodes: Arena<TrieNode>,
    root: Id,
}

impl Trie {
    pub fn new() -> Self {
        let mut nodes = Arena::new();
        let root = nodes.alloc(TrieNode::new(0));
        Self { nodes, root }
    }
}

impl Trie {
    pub fn node(&self, index: Id) -> &TrieNode {
        &self.nodes[index]
    }
    pub fn node_mut(&mut self, index: Id) -> &mut TrieNode {
        &mut self.nodes[index]
    }

    pub fn root(&self) -> Id {
        self.root
    }
    // pub fn parent(&self, index: Id) -> usize {
    //     self.node(index).parent
    // }
    pub fn left(&self, index: Id) -> Id {
        self.node(index).left
    }
    pub fn right(&self, index: Id) -> Id {
        self.node(index).right
    }
    // pub fn is_end(&self, index: Id) -> bool {
    //     self.node(index).is_end
    // }

    pub fn len(&self) -> usize {
        self.nodes.len() - 1
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, word: impl IntoIterator<Item = bool>) -> Id {
        let mut current = self.root;
        for bit in word.into_iter() {
            current = if bit {
                if self.nodes[current].right == 0 {
                    self.nodes[current].right = self.nodes.alloc(TrieNode::new(current));
                }
                self.nodes[current].right
            } else {
                if self.nodes[current].left == 0 {
                    self.nodes[current].left = self.nodes.alloc(TrieNode::new(current));
                }
                self.nodes[current].left
            };
        }
        // self.nodes[current].is_end = true;
        current
    }

    // pub fn contains(&self, word: &[bool]) -> bool {
    //     let mut current = self.root;
    //     for &bit in word {
    //         current = if bit { self.right(current) } else { self.left(current) };
    //         if current == 0 {
    //             return false;
    //         }
    //     }
    //     self.nodes[current].is_end
    // }

    pub fn search(&self, word: &[bool]) -> Id {
        let mut current = self.root;
        for &bit in word {
            current = if bit { self.right(current) } else { self.left(current) };
            if current == 0 {
                return 0;
            }
        }
        current
    }

    // pub(crate) fn level(&self, index: usize) -> usize {
    //     let mut i = 0;
    //     let mut current = index;
    //     loop {
    //         let p = self.parent(current);
    //         if p == 0 {
    //             break;
    //         }
    //         current = p;
    //         i += 1;
    //     }
    //     i
    // }

    pub fn num_leaves(&self) -> usize {
        self.nodes.iter().filter(|node| node.left == 0 && node.right == 0).count()
    }
}

impl Index<Id> for Trie {
    type Output = TrieNode;

    fn index(&self, index: Id) -> &Self::Output {
        self.node(index)
    }
}

impl IndexMut<Id> for Trie {
    fn index_mut(&mut self, index: Id) -> &mut Self::Output {
        self.node_mut(index)
    }
}

pub fn build_trie(cubes: &[Vec<bool>]) -> Trie {
    let mut trie = Trie::new();
    for cube in cubes {
        trie.insert(cube.clone());
    }
    trie
}

impl Trie {
    pub fn iter(&self) -> TrieIter<'_> {
        TrieIter { trie: self, current: None }
    }
}

pub struct TrieIter<'a> {
    trie: &'a Trie,
    current: Option<Vec<bool>>,
}

impl<'a> Iterator for TrieIter<'a> {
    type Item = Vec<bool>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_none() {
            let mut word = Vec::new();
            let mut node = self.trie.root();
            loop {
                let left = self.trie.left(node);
                if left == 0 {
                    let right = self.trie.right(node);
                    if right == 0 {
                        break;
                    } else {
                        word.push(true);
                        node = right;
                    }
                } else {
                    word.push(false);
                    node = left;
                }
            }
            // log::info!("First leaf: {}", node);
            // log::info!("First word: {:?}", word);
            self.current = Some(word);
        } else {
            let mut current = self.current.take().unwrap();
            assert!(!current.is_empty());
            loop {
                // Ascend:
                if let Some(b) = current.pop() {
                    if b {
                        continue;
                    }
                    let node = self.trie.search(&current);
                    let right = self.trie.right(node);
                    if right != 0 {
                        current.push(true);
                        // Descend:
                        let mut node = right;
                        loop {
                            let left = self.trie.left(node);
                            if left != 0 {
                                current.push(false);
                                node = left;
                            } else {
                                let right = self.trie.right(node);
                                if right != 0 {
                                    current.push(true);
                                    node = right;
                                } else {
                                    break;
                                }
                            }
                        }
                        break;
                    }
                } else {
                    // log::warn!("stop iteration");
                    return None;
                }
            }
            self.current = Some(current);
        }

        // log::info!("TrieIter::next() -> {:?}", self.current);
        self.current.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_trie() {
        let trie = Trie::new();
        assert_eq!(trie.nodes.len(), 1);
        assert_ne!(trie.root, 0);
        // assert!(!trie.nodes[trie.root].is_end);
        assert_eq!(trie.left(trie.root), 0);
        assert_eq!(trie.right(trie.root), 0);
    }

    #[test]
    fn test_insert_single_word() {
        let mut trie = Trie::new();
        trie.insert([true, false, true]);

        assert_eq!(trie.left(trie.root), 0);
        assert_ne!(trie.right(trie.root), 0);
        // assert!(!trie.is_end(trie.root));

        assert_eq!(trie.right(trie.right(trie.root)), 0);
        assert_ne!(trie.left(trie.right(trie.root)), 0);
        // assert!(!trie.is_end(trie.right(trie.root)));

        assert_eq!(trie.left(trie.left(trie.right(trie.root))), 0);
        assert_ne!(trie.right(trie.left(trie.right(trie.root))), 0);
        // assert!(!trie.is_end(trie.left(trie.right(trie.root))));

        assert_eq!(trie.left(trie.right(trie.left(trie.right(trie.root)))), 0);
        assert_eq!(trie.right(trie.right(trie.left(trie.right(trie.root)))), 0);
        // assert!(trie.is_end(trie.right(trie.left(trie.right(trie.root)))));
    }

    #[test]
    fn test_insert_multiple_words() {
        let mut trie = Trie::new();
        let a = trie.insert([true, false, true]);
        let b = trie.insert([false, true, false]);
        let c = trie.insert([true, true, true]);
        // assert!(!trie.is_end(trie.root));
        // assert!(trie.is_end(a));
        // assert!(trie.is_end(b));
        // assert!(trie.is_end(c));
        assert_eq!(a, trie.search(&[true, false, true]));
        assert_eq!(b, trie.search(&[false, true, false]));
        assert_eq!(c, trie.search(&[true, true, true]));
    }

    #[test]
    fn test_build_trie() {
        let cubes = vec![vec![true, false, true], vec![false, true, false], vec![true, true, true]];
        let trie = build_trie(&cubes);
        for cube in cubes.iter() {
            // assert!(trie.contains(cube));
            assert_ne!(trie.search(cube), 0);
        }
    }
}
