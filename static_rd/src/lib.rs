#![feature(get_mut_unchecked)]

pub mod trace;
pub use stack_alg_sim::{
    olken::LRUSplay, scale_tree::LRUSplay as LRUScaleTree, stack::LRUStack, vec::LRUVec, LRU,
};
