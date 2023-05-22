use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Each loop and statement is a node in a loop tree.
pub enum LoopTNode {
    /// A single loop
    Loop(SingleLoop),
    /// A statement is a sequence of array references
    Ref(RefStmt),
}

pub type LTNodesRef = RefCell<Vec<Rc<LoopTNode>>>;
pub type LTNodeWeakRef = RefCell<Weak<LoopTNode>>;

pub struct SingleLoop {
    iv: String,
    lb: LoopBound,
    ub: LoopBound,
    // The two arguments are index and upper bound
    test: fn(i32, i32) -> bool,
    step: fn(i32) -> i32,
    body: LTNodesRef,
    parent: LTNodeWeakRef,
}

pub struct RefStmt {
    refs: Vec<ArrRef>,
    parent: LTNodeWeakRef,
}


pub enum LoopBound {
    Fixed(i32),
    Dynamic(fn(IterVec) -> i32),
}

pub struct ArrRef {
    name: String,
    /// Subscript expressions: one function for each data dimension.  
    /// Each function takes the indices of its loop nest and returns indices 
    /// of the array access.
    subexprs: Vec<fn(IterVec) -> ArrAcc>,
}

pub type IterVec = Vec<i32>;
pub type ArrAcc = Vec<i32>;

impl LoopTNode {
    fn sanity(&self) -> i32 {
        match self {
            //    The body of a loop is a vector of LoopTNode's, so we need to
            //    iterate over the vector and sum the sanity of each node.
            LoopTNode::Loop(a_loop) => a_loop.body.borrow().iter().fold(0, |acc, x| acc + x.sanity()),
            LoopTNode::Ref(_) => 1,
        }
    }
}

impl RefStmt {
    fn my_nest(&self) -> LoopTNode {
        // follow the parent pointers and return all enclosing loops as a LoopTNode
        let mut parent = self.parent.borrow().upgrade();
        while let Some(p) = parent {
            // let p = p.clone();
            match p {
                LoopTNode::Loop(_) => return p,
                LoopTNode::Ref(_) => parent = p.parent.borrow().upgrade(),
            }
        }
        unreachable!("No enclosing loop found!")

    }
}


#[cfg(test)]
mod tests {
    use crate::looptnode::LoopTNode::Ref;

    use super::*;

    #[test]
    fn sanity() {
        let aref = RefStmt { refs: Vec::new(), parent: Weak::new().into() };
        let node = Ref(aref);
        assert_eq!(node.sanity(), 1);
    }

    #[test]
    fn matmul() {

    }
}
