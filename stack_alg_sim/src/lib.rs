#![feature(linked_list_remove)]
pub mod olken;
pub mod stack;
pub mod vec;

pub trait LRU<T> {
    fn rec_access(&mut self, val: T) -> Option<usize>;
}
