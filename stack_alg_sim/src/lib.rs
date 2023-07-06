#![feature(linked_list_remove)]
#![feature(let_chains)]
pub mod olken;
pub mod scale_tree;
pub mod stack;
pub mod vec;

pub trait LRU<T> {
    fn rec_access(&mut self, val: T) -> Option<usize>;
}
