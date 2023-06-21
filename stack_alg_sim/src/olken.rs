use fxhash::FxHashMap;
use std::{hash::Hash, ptr::NonNull};

struct SplayNode {
    parent: Option<NonNull<Self>>,
    children: [Option<NonNull<Self>>; 2],
    count: usize,
}

impl SplayNode {
    fn empty() -> Self {
        Self {
            parent: None,
            children: [None, None],
            count: 1,
        }
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
        node.as_mut().count = 1 + Self::left_count(node) + Self::right_count(node);
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
    unsafe fn remove_root(node: NonNull<Self>) -> Option<NonNull<Self>> {
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
    unsafe fn insert_front(node: NonNull<Self>, root: Option<NonNull<Self>>) -> NonNull<Self> {
        if let Some(root) = root {
            let mut front = Self::find_leftmost(root);
            Self::splay(front);
            Self::set_child(node, 1, Some(front));
            front.as_mut().parent = Some(node);
            Self::maintain(node);
        }
        node
    }
}

pub struct LRUSplay<A> {
    root: Option<NonNull<SplayNode>>,
    handles: FxHashMap<A, NonNull<SplayNode>>,
}

impl<A> Default for LRUSplay<A>
where
    A: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A> LRUSplay<A>
where
    A: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            root: None,
            handles: FxHashMap::default(),
        }
    }
    pub fn access(&mut self, key: A) -> Option<usize> {
        unsafe {
            if let Some(node) = self.handles.get(&key) {
                SplayNode::splay(*node);
                let distance = SplayNode::left_count(*node);
                let new_root = SplayNode::remove_root(*node);
                let mut node = *node;
                node.as_mut().count = 1;
                node.as_mut().children = [None, None];
                self.root = Some(SplayNode::insert_front(node, new_root));
                Some(distance + 1)
            } else {
                let node = NonNull::new_unchecked(Box::leak(Box::new(SplayNode::empty())));
                self.handles.insert(key.clone(), node);
                self.root = Some(SplayNode::insert_front(node, self.root));
                None
            }
        }
    }
}

impl<T: Eq + Hash + Clone> crate::LRU<T> for LRUSplay<T> {
    fn rec_access(&mut self, val: T) -> Option<usize> {
        self.access(val)
    }
}

impl<A> Drop for LRUSplay<A> {
    fn drop(&mut self) {
        for (_, node) in self.handles.drain() {
            unsafe {
                let _ = Box::from_raw(node.as_ptr());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LRU;

    #[test]
    fn cyclic() {
        let mut analyzer = LRUSplay::<String>::new();
        let mut dists = Vec::new();
        // let st = "abc abc";
        for c in "abc abc".chars().filter(|c| !c.is_whitespace()) {
            dists.push(analyzer.rec_access(c.to_string()));
        }

        assert_eq!(dists, [None, None, None, Some(3), Some(3), Some(3)]);
    }

    #[test]
    fn cyclic_slice() {
        let mut analyzer = LRUSplay::<&str>::new();
        let mut dists = Vec::new();
        let st = "abcabc";
        for i in 0..st.len() {
            dists.push(analyzer.rec_access(&st[i..i + 1]));
        }

        assert_eq!(dists, [None, None, None, Some(3), Some(3), Some(3)]);
    }

    #[test]
    fn cyclic_large() {
        #[cfg(miri)]
        let limit = 64;
        #[cfg(not(miri))]
        let limit = 10000;

        let mut analyzer = LRUSplay::new();
        let mut dists = Vec::new();
        for c in (0..limit).chain(0..limit) {
            dists.push(analyzer.rec_access(c));
        }
        let access: Vec<Option<usize>> = (0..limit)
            .map(|_| None)
            .chain((0..limit).map(|_| Some(limit)))
            .collect();
        assert_eq!(&dists, &access);
    }

    #[test]
    fn sawtooth() {
        let mut analyzer = LRUSplay::<String>::new();
        let mut dists = Vec::new();
        for c in "abc cba".chars().filter(|c| !c.is_whitespace()) {
            dists.push(analyzer.rec_access(c.to_string()));
        }

        assert_eq!(dists, [None, None, None, Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn sawtooth_large() {
        #[cfg(miri)]
        let limit = 64;
        #[cfg(not(miri))]
        let limit = 10000;

        let mut analyzer = LRUSplay::new();
        let mut dists = Vec::new();
        for c in (0..limit).chain((0..limit).rev()) {
            dists.push(analyzer.rec_access(c));
        }
        let access: Vec<Option<usize>> = (0..limit)
            .map(|_| None)
            .chain((0..limit).map(|x| Some(x + 1)))
            .collect();
        assert_eq!(&dists, &access);
    }
}
