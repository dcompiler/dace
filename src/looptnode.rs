use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Each loop and statement is a node in a loop tree.
pub struct LoopTNode {
    stmt: Stmt,
    pub parent: LTNodeWeakRef
}

pub enum Stmt {
    /// A single loop
    Loop(LoopStmt),
    /// A statement is a sequence of array references
    Ref(RefStmt),
}

pub type LTNodesRef = RefCell<Vec<Rc<LoopTNode>>>;
pub type LTNodeWeakRef = RefCell<Weak<LoopTNode>>;

pub struct LoopStmt {
    pub iv: String,
    pub lb: LoopBound,
    pub ub: LoopBound,
    // The next two need the FnOnce trait, which we'll add later
    // Now we assume test is iv < ub
    // pub test: fn(i32) -> bool,
    // Now we assume step is iv = iv + 1
    // pub step: fn(i32) -> i32,
    pub body: LTNodesRef,
}

pub struct RefStmt {
    pub refs: Vec<ArrRef>,
}


pub enum LoopBound {
    Fixed(i32),
    Dynamic(fn(IterVec) -> i32),
}

#[derive(Clone)]
pub struct ArrRef {
    name: String,
    /// Subscript expressions: one function for each data dimension.  
    /// Each function takes the indices of its loop nest and returns indices 
    /// of the array access.
    pub sub: fn(IterVec)->ArrAcc,
}

pub type IterVec = Vec<i32>;
pub type ArrAcc = Vec<i32>;

impl Stmt {
    fn sanity(&self) -> i32 {
        match self {
            //    The body of a loop is a vector of LoopTNode's, so we need to
            //    iterate over the vector and sum the sanity of each node.
            Stmt::Loop(a_loop) => a_loop.body.borrow().iter().fold(0, |acc, x| acc + x.stmt.sanity()),
            Stmt::Ref(_) => 1,
        }
    }
}

impl RefStmt {
    // fn my_nest(&self) -> LoopTNode {
    //     // follow the parent pointers and return all enclosing loops as a LoopTNode
    //     // let mut parent = self.parent.borrow().upgrade();
    //     // while let Some(p) = parent {
    //     //     match p {
    //     //         LoopTNode::Loop(_) => return p,
    //     //         LoopTNode::Ref(_) => parent = p.parent.borrow().upgrade(),
    //     //     }
    //     // }
    //     // unreachable!("No enclosing loop found!")
    // }
}


#[cfg(test)]
mod tests {
    use crate::looptnode::*;

    #[test]
    fn stmt_sanity() {
        let aref = RefStmt { refs: Vec::new() };
        let stmt = Stmt::Ref(aref);
        assert_eq!(stmt.sanity(), 1);
    }

    #[test]
    fn acc_ref() {
        let ar = ArrRef{name: "X".to_string(), sub: |iv| vec![iv[0]+1] };
        assert_eq!((ar.sub)(vec![1]), [2]);
    }

    #[test]
    fn matmul() {
        let n = 100;
	// creating C[i,j] += A[i,k] * B[k,j]
        let c = ArrRef{ name: "C".to_string(), sub: |ijk| vec![ijk[0], ijk[1]] };
	let a = ArrRef{ name: "A".to_string(), sub: |ijk| vec![ijk[0], ijk[2]] };
	let b = ArrRef{ name: "B".to_string(), sub: |ijk| vec![ijk[2], ijk[1]] };
	let s = RefStmt{ refs: vec![c.clone(), c, a, b] };
	// creating loop k = 0, n { s_ref }
	let s_ref = Rc::new(LoopTNode{ stmt: Stmt::Ref(s),
					       parent: RefCell::new(Weak::new()) });
	let k_loop = LoopStmt{ iv: "k".to_string(),
			       lb: LoopBound::Fixed(0), ub: LoopBound::Fixed(n),
			       // test: |k| k<n , step: |k| k+1,
			       body: RefCell::new(vec![s_ref]) };
	// officiating parent-child relationship
	// (*k_loop.body)[0].borrow_mut().parent = RefCell::new(Rc::downgrade(Rc::new(k_loop)));
	
    }
}
