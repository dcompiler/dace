use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Each loop and statement is a node in a loop tree.
pub struct LoopTNode {
    pub stmt: Stmt,
    parent: LTNodeWeakRef
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

impl LoopTNode {
    fn new_ref(a_stmt: Stmt) -> Rc<LoopTNode> {
	return Rc::new( LoopTNode{ stmt: a_stmt,
			    parent: RefCell::new(Weak::new())
	});
    }

    fn new_simple_loop_ref( ivar: &str, low: i32, high: i32 ) -> Rc<LoopTNode> {
	let loop_stmt = LoopStmt{ iv: ivar.to_string(), 
				    lb: LoopBound::Fixed(low), ub: LoopBound::Fixed(high),
				    // test: |k| k<n , step: |k| k+1,
				  body: RefCell::new(vec![]) };
	return LoopTNode::new_ref( Stmt::Loop(loop_stmt) );
    }
    
    fn extend_loop_body(loop_ref: &Rc<LoopTNode>, stmt_ref: &Rc<LoopTNode>) {
	if let Stmt::Loop(ref lp) = loop_ref.stmt {
	    // officiating the parent-child relationship	    
	    *stmt_ref.parent.borrow_mut() = Rc::downgrade(loop_ref);
	    // adding to the body
	    lp.body.borrow_mut().push(Rc::clone(stmt_ref));
	}
	else {
	    panic!("extend_loop_body called on non-loop node");
	}
    }

    fn node_count( &self ) -> u32 {
        match & self.stmt {
            //    The body of a loop is a vector of LoopTNode's, so we need to
            //    iterate over the vector and sum the sanity of each node.
            Stmt::Loop(a_loop) => a_loop.body.borrow().iter().fold(1, |acc, x| acc + x.node_count()),
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
	let s_ref = LoopTNode::new_ref( Stmt::Ref(s) );
	// creating loop k = 0, n { s_ref }
	let k_loop_ref = LoopTNode::new_simple_loop_ref( "k", 0, n );
	LoopTNode::extend_loop_body( &k_loop_ref, &s_ref );
	// creating loop j = 0, n
	let j_loop_ref = LoopTNode::new_simple_loop_ref( "j", 0, n );
	LoopTNode::extend_loop_body( &j_loop_ref, &k_loop_ref );
	// creating loop i = 0, n
	let i_loop_ref = LoopTNode::new_simple_loop_ref( "i", 0, n );
	LoopTNode::extend_loop_body( &i_loop_ref, &j_loop_ref );

	assert_eq!(i_loop_ref.node_count(), 4);
	
	// let s_ref = Rc::new(LoopTNode{ stmt: Stmt::Ref(s),
	// 			       parent: RefCell::new(Weak::new()) });
	// let k_loop_stmt = LoopStmt{ iv: "k".to_string(), 
	// 			    lb: LoopBound::Fixed(0), ub: LoopBound::Fixed(n),
	// 			    // test: |k| k<n , step: |k| k+1,
	// 			    body: RefCell::new(vec![]) };
	// let k_loop_ref = LoopTNode::new_ref( Stmt::Loop(k_loop_stmt) );
	// let k_loop_ref = Rc::new(
	//     LoopTNode{ stmt: Stmt::Loop(k_loop_stmt),
	// 	       parent: RefCell::new(Weak::new())
	//     });
	// // officiating the parent-child relationship
	// if let Stmt::Loop(ref lp) = k_loop_ref.stmt {
	//     *(lp.body.borrow())[0].parent.borrow_mut() = Rc::downgrade(&k_loop_ref);
	// }
	// let j_loop_stmt = LoopStmt{ iv: "j".to_string(),
	// 			    lb: LoopBound::Fixed(0), ub: LoopBound::Fixed(n),
	// 			    body: RefCell::new(vec![]) };
	// let j_loop_ref = LoopTNode::new_ref( Stmt::Loop(j_loop_stmt) );
	// let j_loop_ref = Rc::new(
	//     LoopTNode{ stmt: Stmt::Loop(j_loop_stmt),
	// 	       parent: RefCell::new(Weak::new())
	//     });
	// if let Stmt::Loop(ref lp) = j_loop_ref.stmt {
	//     *(lp.body.borrow())[0].parent.borrow_mut() = Rc::downgrade(&j_loop_ref);
	// }
	// let i_loop_stmt = LoopStmt{ iv: "i".to_string(),
	// 			    lb: LoopBound::Fixed(0), ub: LoopBound::Fixed(n),
	// 			    body: RefCell::new(vec![]) };
	// let i_loop_ref = LoopTNode::new_ref( Stmt::Loop(i_loop_stmt) );
	// let i_loop_ref = Rc::new(
	//     LoopTNode{ stmt: Stmt::Loop(i_loop_stmt),
	// 	       parent: RefCell::new(Weak::new())
	//     });
	// if let Stmt::Loop(ref lp) = i_loop_ref.stmt {
	//     *(lp.body.borrow())[0].parent.borrow_mut() = Rc::downgrade(&i_loop_ref);
	// }

    }
}
