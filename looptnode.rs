use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Each loop and statement is a node in a loop tree.
pub enum LoopTNode {
    /// A single loop
    Loop {
        iv: String,
        lb: LoopBound,
        ub: LoopBound,
        // The two arguments are index and upper bound
        test: fn(i32, i32)->bool,
        step: fn(i32)->i32,
        body: LTNodesRef,
        parent: LTNodeWeakRef
    },
    /// A statement is a sequence of array references
    RefStmt {
        refs: Vec<ArrRef>,
        parent: LTNodeWeakRef
    }
}

pub type LTNodesRef = RefCell<Vec<Rc<LoopTNode>>>;
pub type LTNodeWeakRef = RefCell<Weak<LoopTNode>>;

pub enum LoopBound {
    Fixed(i32),
    Dynamic(fn(IterVec)->i32)
}

pub struct ArrRef {
    name: String,
    /// Subscript expressions: one function for each data dimension.  
    /// Each function takes the indices of its loop nest and returns indices 
    /// of the array access.
    subexprs: Vec<fn(IterVec) -> ArrAcc>
}

pub type IterVec = Vec<i32>;
pub type ArrAcc = Vec<i32>;

