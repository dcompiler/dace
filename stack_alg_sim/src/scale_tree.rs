use fxhash::FxHashMap;
use std::cell::UnsafeCell;
use std::collections::hash_map::Entry::*;
use std::{
    hash::Hash,
    ptr::NonNull,
    rc::{Rc, Weak},
};
struct SplayNode {
    parent: Option<NonNull<Self>>,
    children: [Option<NonNull<Self>>; 2],
    merged: Option<Rc<UnsafeCell<Self>>>,
    this: Weak<UnsafeCell<Self>>,
    size: usize,
    count: usize,
}

impl SplayNode {
    fn empty() -> Rc<UnsafeCell<Self>> {
        Rc::new_cyclic(|this| {
            UnsafeCell::new(Self {
                parent: None,
                children: [None, None],
                merged: None,
                this: this.clone(),
                size: 1,
                count: 1,
            })
        })
    }
    // find a position where distance to the leftmost is at most `threshold`
    unsafe fn cut_position(
        node: NonNull<Self>,
        threshold: usize,
    ) -> Result<NonNull<Self>, NonNull<Self>> {
        let mut result = node;
        while let Some(x) = Self::get_child(result, 1) && result.as_ref().count > threshold {
            result = x;
        }
        if result.as_ref().count > threshold {
            Err(result)
        } else {
            Ok(result)
        }
    }

    unsafe fn expose(
        mut node: NonNull<Self>,
        error_bound: f64,
        compression_threshold: usize,
    ) -> NonNull<Self> {
        Self::splay(node);
        // check marker
        while let Some(merged) = node.as_ref().merged.as_ref() {
            let old = node;
            node = NonNull::new_unchecked(merged.get());
            Self::splay(node);
            if let Some(mut new_root) = Self::remove_root_node(old) {
                // pass down marker (with path halving)
                new_root.as_mut().merged = Some(
                    node.as_ref()
                        .merged
                        .clone()
                        .unwrap_or_else(|| merged.clone()),
                );
            }
        }
        // compress
        Self::compress_root(node, error_bound, compression_threshold);
        node
    }
    unsafe fn remove_root(mut node: NonNull<Self>) -> Option<Option<NonNull<Self>>> {
        if node.as_ref().size > 1 {
            node.as_mut().size -= 1;
            Self::maintain(node);
            return None;
        }
        Some(Self::remove_root_node(node))
    }
    // access will make a node the root of the tree.
    unsafe fn access(
        node: NonNull<Self>,
        error_bound: f64,
        compression_threshold: usize,
    ) -> (usize, Rc<UnsafeCell<Self>>) {
        let mut node = Self::expose(node, error_bound, compression_threshold);
        let order = node.as_ref().count - Self::right_count(node);
        let handle = match Self::remove_root(node) {
            None => {
                let handle = SplayNode::empty();
                let new_root = NonNull::new_unchecked(handle.get());
                Self::set_child(new_root, 1, Some(node));
                node.as_mut().parent = Some(new_root);
                Self::maintain(new_root);
                handle
            }
            Some(root) => {
                // node adopt root as right child
                Self::set_child(node, 0, None);
                Self::set_child(node, 1, root);
                if let Some(mut root) = root {
                    root.as_mut().parent = Some(node);
                }
                Self::maintain(node);
                node.as_ref().this.upgrade().unwrap_unchecked()
            }
        };
        (order, handle)
    }
    unsafe fn compress_root(
        mut node: NonNull<Self>,
        error_bound: f64,
        compression_threshold: usize,
    ) {
        let distance = node.as_ref().count - Self::right_count(node);
        let capacity = ((distance as f64 * ((error_bound) / (1.0 - error_bound))) as usize).max(1);
        if capacity - node.as_ref().size < compression_threshold {
            return;
        }
        if let Some(mut left) = Self::get_child(node, 0) {
            Self::set_child(node, 0, None);
            left.as_mut().parent = None;
            match Self::cut_position(left, capacity - node.as_ref().size) {
                Ok(mut cut_position) => {
                    if cut_position != left {
                        // splay parent of cut_position to root
                        let mut root = cut_position.as_mut().parent.unwrap_unchecked();
                        Self::splay(root);
                        cut_position = root.as_ref().children[1].unwrap_unchecked();

                        // disconnect cut_position from root
                        Self::set_child(root, 1, None);
                        Self::maintain(root);

                        // reconnect root to node
                        Self::set_child(node, 0, Some(root));
                        root.as_mut().parent = Some(node);
                    }
                    cut_position.as_mut().parent = None;
                    node.as_mut().size += cut_position.as_ref().count;
                    cut_position.as_mut().merged =
                        Some(node.as_ref().this.upgrade().unwrap_unchecked());
                }
                Err(mut last) => {
                    // splay last to amortize the query cost
                    Self::splay(last);
                    Self::set_child(node, 0, Some(last));
                    last.as_mut().parent = Some(node);
                }
            }
        }
        //Self::assert_tree_correctness(node);
    }
    #[inline(always)]
    unsafe fn left_count(node: NonNull<Self>) -> usize {
        node.as_ref().children[0]
            .map(|x| x.as_ref().count)
            .unwrap_or(0)
    }
    #[inline(always)]
    unsafe fn right_count(node: NonNull<Self>) -> usize {
        node.as_ref().children[1]
            .map(|x| x.as_ref().count)
            .unwrap_or(0)
    }
    #[inline(always)]
    unsafe fn maintain(mut node: NonNull<Self>) {
        node.as_mut().count = node.as_ref().size + Self::left_count(node) + Self::right_count(node);
    }
    #[inline(always)]
    unsafe fn get_child(node: NonNull<Self>, dir: usize) -> Option<NonNull<Self>> {
        node.as_ref().children[dir]
    }
    #[inline(always)]
    unsafe fn set_child(mut node: NonNull<Self>, dir: usize, child: Option<NonNull<Self>>) {
        node.as_mut().children[dir] = child;
    }
    #[inline(always)]
    unsafe fn is_right_child(node: NonNull<Self>) -> bool {
        node.as_ref().parent.unwrap_unchecked().as_ref().children[1] == Some(node)
    }
    unsafe fn rotate(mut node: NonNull<Self>) {
        debug_assert!(node.as_ref().parent.is_some());

        let mut parent = node.as_ref().parent.unwrap_unchecked();
        let is_right = Self::is_right_child(node);

        // update grandparent
        // there is no need to maintain node, the count is not changed
        if let Some(grandparent) = parent.as_ref().parent {
            Self::set_child(
                grandparent,
                Self::is_right_child(parent) as usize,
                Some(node),
            );
        }
        node.as_mut().parent = parent.as_ref().parent;

        // hand over child
        let target_child = Self::get_child(node, (!is_right) as usize);
        Self::set_child(parent, is_right as usize, target_child);
        if let Some(mut child) = target_child {
            child.as_mut().parent = Some(parent);
        }
        Self::maintain(parent);

        // adopt parent
        Self::set_child(node, (!is_right) as usize, Some(parent));
        node.as_mut().merged = parent.as_mut().merged.take();
        parent.as_mut().parent = Some(node);
        Self::maintain(node);
    }
    unsafe fn splay(node: NonNull<Self>) {
        while let Some(parent) = node.as_ref().parent {
            if parent.as_ref().parent.is_some() {
                if Self::is_right_child(node) == Self::is_right_child(parent) {
                    Self::rotate(parent);
                } else {
                    Self::rotate(node);
                }
            }
            Self::rotate(node);
        }
    }
    unsafe fn find_leftmost(mut node: NonNull<Self>) -> NonNull<Self> {
        while let Some(child) = Self::get_child(node, 0) {
            node = child;
        }
        node
    }
    unsafe fn remove_root_node(node: NonNull<Self>) -> Option<NonNull<Self>> {
        let left = Self::get_child(node, 0);
        let right = Self::get_child(node, 1);
        if let Some(mut right) = right {
            // separate right
            right.as_mut().parent = None;
            let leftmost = Self::find_leftmost(right);
            Self::splay(leftmost);
            Self::set_child(leftmost, 0, left);
            if let Some(mut left) = left {
                left.as_mut().parent = Some(leftmost);
            }
            Self::maintain(leftmost);
            Some(leftmost)
        } else {
            // separate left
            if let Some(mut left) = left {
                left.as_mut().parent = None;
            }
            left
        }
    }
}

impl<T: Eq + Hash + Clone> crate::LRU<T> for LRUSplay<T> {
    fn rec_access(&mut self, val: T) -> Option<usize> {
        self.access(val)
    }
}

pub struct LRUSplay<A> {
    root: Option<Rc<UnsafeCell<SplayNode>>>,
    handles: FxHashMap<A, Rc<UnsafeCell<SplayNode>>>,
    error_bound: f64,
    compression_threshold: usize,
}

impl<A> Default for LRUSplay<A>
where
    A: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new(0.0, usize::MAX)
    }
}

impl<A> LRUSplay<A>
where
    A: Eq + Hash + Clone,
{
    pub fn new(error_bound: f64, compression_threshold: usize) -> Self {
        Self {
            root: None,
            handles: FxHashMap::default(),
            error_bound,
            compression_threshold,
        }
    }
    pub fn access(&mut self, key: A) -> Option<usize> {
        unsafe {
            match self.handles.entry(key) {
                Occupied(mut entry) => {
                    let node = entry.get().clone();
                    let node = NonNull::new_unchecked(node.get());
                    let (count, handle) =
                        SplayNode::access(node, self.error_bound, self.compression_threshold);
                    self.root = Some(handle.clone());
                    entry.insert(handle);
                    Some(count)
                }
                Vacant(entry) => {
                    let handle = SplayNode::empty();
                    let node = NonNull::new_unchecked(handle.get());
                    let root = self.root.clone().map(|x| NonNull::new_unchecked(x.get()));
                    // adopt root
                    if let Some(mut root) = root {
                        root.as_mut().parent = Some(node);
                    }
                    SplayNode::set_child(node, 1, root);
                    SplayNode::maintain(node);
                    self.root = Some(handle.clone());
                    entry.insert(handle);
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyclic() {
        let mut analyzer = LRUSplay::default();
        let mut dists = Vec::new();
        // let st = "abc abc";
        for c in "abc abc".chars().filter(|c| !c.is_whitespace()) {
            dists.push(analyzer.access(c.to_string()));
        }

        assert_eq!(dists, [None, None, None, Some(3), Some(3), Some(3)]);
    }

    #[test]
    fn cyclic_slice() {
        let mut analyzer = LRUSplay::default();
        let mut dists = Vec::new();
        let st = "abcabc";
        for i in 0..st.len() {
            dists.push(analyzer.access(&st[i..i + 1]));
        }

        assert_eq!(dists, [None, None, None, Some(3), Some(3), Some(3)]);
    }

    #[test]
    fn cyclic_large() {
        #[cfg(miri)]
        let limit = 64;
        #[cfg(not(miri))]
        let limit = 100000;

        let mut analyzer = LRUSplay::default();
        let mut dists = Vec::new();
        for c in (0..limit).chain(0..limit) {
            dists.push(analyzer.access(c));
        }
        let access: Vec<Option<usize>> = (0..limit)
            .map(|_| None)
            .chain((0..limit).map(|_| Some(limit)))
            .collect();
        assert_eq!(&dists, &access);
    }

    #[test]
    fn sawtooth() {
        let mut analyzer = LRUSplay::default();
        let mut dists = Vec::new();
        for c in "abc cba".chars().filter(|c| !c.is_whitespace()) {
            dists.push(analyzer.access(c.to_string()));
        }

        assert_eq!(dists, [None, None, None, Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn sawtooth_large() {
        #[cfg(miri)]
        let limit = 64;
        #[cfg(not(miri))]
        let limit = 100000;

        let mut analyzer = LRUSplay::default();
        let mut dists = Vec::new();
        for c in (0..limit).chain((0..limit).rev()) {
            dists.push(analyzer.access(c));
        }
        let access: Vec<Option<usize>> = (0..limit)
            .map(|_| None)
            .chain((0..limit).map(|x| Some(x + 1)))
            .collect();
        assert_eq!(&dists, &access);
    }
}
