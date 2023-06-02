use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Each loop and statement is a node in a loop tree.
pub struct LoopTNode {
    pub stmt: Stmt,
    parent: LTNodeWeakRef,
}

pub type LTNodesRef = RefCell<Vec<Rc<LoopTNode>>>;
pub type LTNodeWeakRef = RefCell<Weak<LoopTNode>>;

/// Statements in the loop tree.
pub enum Stmt {
    /// A single loop
    Loop(LoopStmt),
    /// A statement is a sequence of array references
    // Ref(RefStmt),
    Ref(AryRef),
}

pub struct LoopStmt {
    pub iv: String,
    pub lb: LoopBound,
    pub ub: LoopBound,
    // The next two need the FnOnce trait, which we'll add later
    // Now we assume test is iv < ub
    pub test: fn(i32,i32) -> bool,
    // Now we assume step is iv = iv + 1
    pub step: fn(i32) -> i32,
    pub body: LTNodesRef,
}


pub enum LoopBound {
    Fixed(i32),
    Dynamic(fn(&IterVec) -> i32),
}

// pub struct RefStmt {
//     pub refs: Vec<AryRef>,
// }

/// Array reference.
#[derive(Clone)]
pub struct AryRef {
    pub name: String,
    /// array dimensions, e.g. [5,5]
    pub dim: Vec<usize>,
    /// Subscript expressions: one function for each data dimension.  
    /// Each function takes the indices of its loop nest and returns indices of the array access.
    pub sub: fn(&IterVec) -> AryAcc,
}

/// Type alias for the iteration vector, with i32 elements.
pub type IterVec = Vec<i32>;
/// Type alias for the array access indices, with usize elements.
pub type AryAcc = Vec<usize>;

impl LoopTNode {
    /// Create a new LoopTNode with a given statement.
    pub fn new_node(a_stmt: Stmt) -> Rc<LoopTNode> {
        return Rc::new(LoopTNode {
            stmt: a_stmt,
            parent: RefCell::new(Weak::new()),
        });
    }

    pub fn new_ref(ary_nm: &str, ary_dim: Vec<usize>,
		   ary_sub: fn(&Vec<i32>)->Vec<usize>) -> Rc<LoopTNode> {
	let ref_stmt = AryRef { name: ary_nm.to_string(), dim: ary_dim, sub: ary_sub };
	LoopTNode::new_node(Stmt::Ref(ref_stmt))
    }

    /// Create a new LoopTNode representing a simple loop with a fixed range.
    pub fn new_single_loop(ivar: &str, low: i32, high: i32) -> Rc<LoopTNode> {
        let loop_stmt = LoopStmt {
            iv: ivar.to_string(),
            lb: LoopBound::Fixed(low),
            ub: LoopBound::Fixed(high),
            // test: |i| i<ub , step: |k| k+1,
            test: |i, ub| i < ub,
            step: |i| i + 1,
            body: RefCell::new(vec![]),
        };
        LoopTNode::new_node(Stmt::Loop(loop_stmt))
    }

    /// Extend the body of a loop node with another node.
    pub fn extend_loop_body(loop_ref: &Rc<LoopTNode>, stmt_ref: &Rc<LoopTNode>) {
        if let Stmt::Loop(ref lp) = loop_ref.stmt {
            // officiating the parent-child relationship
            *stmt_ref.parent.borrow_mut() = Rc::downgrade(loop_ref);
            // adding to the body
            lp.body.borrow_mut().push(Rc::clone(stmt_ref));
        } else {
            panic!("extend_loop_body called on non-loop node");
        }
    }

    // Get the count of nodes in the loop tree.
    fn node_count(&self) -> u32 {
        match &self.stmt {
            //    The body of a loop is a vector of LoopTNode's, so we need to
            //    iterate over the vector and sum the sanity of each node.
            Stmt::Loop(a_loop) => a_loop.body.borrow().iter().fold(1, |acc, x| acc + x.node_count()),
            Stmt::Ref(_) => 1,
        }
    }
}

// impl RefStmt {
//     fn my_nest(&self) -> LoopTNode {
//         // follow the parent pointers and return all enclosing loops as a LoopTNode
//         // let mut parent = self.parent.borrow().upgrade();
//         // while let Some(p) = parent {
//         //     match p {
//         //         LoopTNode::Loop(_) => return p,
//         //         LoopTNode::Ref(_) => parent = p.parent.borrow().upgrade(),
//         //     }
//         // }
//         // unreachable!("No enclosing loop found!")
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acc_ref() {
        let ar = AryRef { name: "X".to_string(),
			  dim: vec![10],
			  sub: |iv| vec![(iv[0] as usize) + 1] };
        assert_eq!((ar.sub)(&vec![1]), [2]);
    }

    #[test]
    fn matmul() {
        let n: usize = 100;  // array dim
	let ubound = n as i32;  // loop bound
        // creating C[i,j] += A[i,k] * B[k,j]
        let s_ref_c = LoopTNode::new_ref("C", vec![n,n],
					 |ijk| vec![ijk[0] as usize, ijk[1] as usize]);
        let s_ref_a = LoopTNode::new_ref("A", vec![n,n],
					 |ijk| vec![ijk[0] as usize, ijk[2] as usize]);
        let s_ref_b = LoopTNode::new_ref("B", vec![n,n],
					 |ijk| vec![ijk[2] as usize, ijk[1] as usize]);

        // creating loop k = 0, n { s_ref }
        let k_loop_ref = LoopTNode::new_single_loop("k", 0, ubound);
        LoopTNode::extend_loop_body(&k_loop_ref, &s_ref_c);
        LoopTNode::extend_loop_body(&k_loop_ref, &s_ref_a);
        LoopTNode::extend_loop_body(&k_loop_ref, &s_ref_b);
        // creating loop j = 0, n
        let j_loop_ref = LoopTNode::new_single_loop("j", 0, ubound);
        LoopTNode::extend_loop_body(&j_loop_ref, &k_loop_ref);
        // creating loop i = 0, n
        let i_loop_ref = LoopTNode::new_single_loop("i", 0, ubound);
        LoopTNode::extend_loop_body(&i_loop_ref, &j_loop_ref);

        assert_eq!(i_loop_ref.node_count(), 6);

        // let s_ref = Rc::new(LoopTNode{ stmt: Stmt::Ref(s),
        // 			       parent: RefCell::new(Weak::new()) });
        // let k_loop_stmt = LoopStmt{ iv: "k".to_string(),
        // 			    lb: LoopBound::Fixed(0), ub: LoopBound::Fixed(n),
        // 			    // test: |k| k<n , step: |k| k+1,
        // 			    body: RefCell::new(vec![]) };
        // let k_loop_ref = LoopTNode::new_node( Stmt::Loop(k_loop_stmt) );
        // let k_loop_ref = Rc::new(
        //     LoopTNode{ stmt: Stmt::Loop(k_loop_stmt),
        // 	       parent: RefCell::new(Weak::new())
        //     });
        // // officiating the parent-child relationship
        // if let Stmt::Loop(ref lp) = k_loop_ref.stmt {
        //     *(lp.body.borrow())[0].parent.borrow_mut() = Rc::downgrade(&k_loop_ref);
        // }
        // let j_loop_stmt = LoopStmt{ iv: "j".to_string(),
        // 			    lb: LoopBound::Fixed(0), ub: LoopBound::Fixed(ubound),
        // 			    body: RefCell::new(vec![]) };
        // let j_loop_ref = LoopTNode::new_node( Stmt::Loop(j_loop_stmt) );
        // let j_loop_ref = Rc::new(
        //     LoopTNode{ stmt: Stmt::Loop(j_loop_stmt),
        // 	       parent: RefCell::new(Weak::new())
        //     });
        // if let Stmt::Loop(ref lp) = j_loop_ref.stmt {
        //     *(lp.body.borrow())[0].parent.borrow_mut() = Rc::downgrade(&j_loop_ref);
        // }
        // let i_loop_stmt = LoopStmt{ iv: "i".to_string(),
        // 			    lb: LoopBound::Fixed(0), ub: LoopBound::Fixed(ubound),
        // 			    body: RefCell::new(vec![]) };
        // let i_loop_ref = LoopTNode::new_node( Stmt::Loop(i_loop_stmt) );
        // let i_loop_ref = Rc::new(
        //     LoopTNode{ stmt: Stmt::Loop(i_loop_stmt),
        // 	       parent: RefCell::new(Weak::new())
        //     });
        // if let Stmt::Loop(ref lp) = i_loop_ref.stmt {
        //     *(lp.body.borrow())[0].parent.borrow_mut() = Rc::downgrade(&i_loop_ref);
        // }
    }

    #[test]
    fn mat_transpose1() {
        let n: usize = 1024;
	let ubound = n as i32;
        // for (int c0 = 0; c0 < n; c0 += 1)
        // for (int c1 = 0; c1 < n; c1 += 1)
        //   x1[c0] = (x1[c0] + (A[c0][c1] * y_1[c1]));
        let s_ref_x1 = LoopTNode::new_ref("x1", vec![n], |ij| vec![ij[0] as usize]);
        let s_ref_a = LoopTNode::new_ref("a", vec![n,n],
					  |ij| vec![ij[0] as usize, ij[1] as usize]);
        let s_ref_y1 = LoopTNode::new_ref("y1", vec![n], |ij| vec![ij[1] as usize]);

        let j_loop_ref = LoopTNode::new_single_loop("j", 0, ubound);
        LoopTNode::extend_loop_body(&j_loop_ref, &s_ref_x1);
        LoopTNode::extend_loop_body(&j_loop_ref, &s_ref_a);
        LoopTNode::extend_loop_body(&j_loop_ref, &s_ref_y1);

        let i_loop_ref = LoopTNode::new_single_loop("i", 0, ubound);
        LoopTNode::extend_loop_body(&i_loop_ref, &j_loop_ref);

        assert_eq!(i_loop_ref.node_count(), 5);
    }

    // #[test]
    // fn mat_transpose2() {
    //     let n: usize = 1024;
    // 	let ubound = n as i32;
    //     //     for (int c0 = 0; c0 < n; c0 += 1)
    //     //     for (int c1 = 0; c1 < n; c1 += 1)
    //     //     x2[c0] = (x2[c0] + (A[c1][c0] * y_2[c1]));
    //     let x2 = AryRef { name: "x2".to_string(), dim: vec![n], sub: |i| vec![i[0]] };
    //     let a = AryRef { name: "A".to_string(), dim: vec![n,n], sub: |ij| vec![ij[0], ij[1]] };
    //     let y2 = AryRef { name: "y2".to_string(), dim: vec![n], sub: |j| vec![j[0], j[1]] };
    //     // let s = RefStmt{ refs: vec![x2.clone(), x2.clone(), a.clone(), y2.clone()] };
    //     // let s = vec![x2.clone(), x2.clone(), a.clone(), y2.clone()];
    //     // let s_ref = LoopTNode::new_node(Stmt::Ref(s));

    //     let s_ref_x1 = LoopTNode::new_node(Stmt::Ref(x2));
    //     let s_ref_a = LoopTNode::new_node(Stmt::Ref(a));
    //     let s_ref_y1 = LoopTNode::new_node(Stmt::Ref(y2));


    //     let j_loop_ref = LoopTNode::new_single_loop("j", 0, ubound);
    //     LoopTNode::extend_loop_body(&j_loop_ref, &s_ref_x1);
    //     LoopTNode::extend_loop_body(&j_loop_ref, &s_ref_a);
    //     LoopTNode::extend_loop_body(&j_loop_ref, &s_ref_y1);

    //     let i_loop_ref = LoopTNode::new_single_loop("i", 0, ubound);
    //     LoopTNode::extend_loop_body(&i_loop_ref, &j_loop_ref);

    //     assert_eq!(i_loop_ref.node_count(), 5);
    // }
}
