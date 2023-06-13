use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::{Rc, Weak};

/// Each loop and statement is a node in a loop tree.
#[derive(Debug)]
pub struct Node {
    pub stmt: Stmt,
    parent: NodeWeakRef,
}

pub type NodesRef = RefCell<Vec<Rc<Node>>>;
pub type NodeWeakRef = RefCell<Weak<Node>>;

/// Statements in the loop tree.
#[derive(Debug)]
pub enum Stmt {
    /// A single loop
    Loop(LoopStmt),
    /// A statement is a sequence of array references
    // Ref(RefStmt),
    Ref(AryRef),
    Block(Vec<Rc<Node>>),
}

pub struct LoopStmt {
    pub iv: String,
    pub lb: LoopBound,
    pub ub: LoopBound,
    // The next two need the FnOnce trait, which we'll add later
    // Now we assume test is iv < ub
    pub test: Box<dyn Fn(i32, i32) -> bool>,
    // Now we assume step is iv = iv + 1
    pub step: Box<dyn Fn(i32) -> i32>,
    pub body: NodesRef,
}

impl Debug for LoopStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoopStmt")
            .field("iv", &self.iv)
            .field("lb", &self.lb)
            .field("ub", &self.ub)
            .field("body", &self.body)
            .finish_non_exhaustive()
    }
}

pub enum LoopBound {
    Fixed(i32),
    Dynamic(fn(&IterVec) -> i32),
}

impl std::fmt::Debug for LoopBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            LoopBound::Fixed(x) => write!(f, "Fixed({x})"),
            LoopBound::Dynamic(_) => write!(f, "Dynamic"),
        }
    }
}

// pub struct RefStmt {
//     pub refs: Vec<AryRef>,
// }

/// Array reference.
pub struct AryRef {
    pub name: String,
    /// array dimensions, e.g. [5,5]
    pub dim: Vec<usize>,
    /// Subscript expressions: one function for each data dimension.  
    /// Each function takes the indices of its loop nest and returns indices of the array access.
    pub sub: Box<dyn for<'a> Fn(&'a IterVec) -> AryAcc>,
    pub base: Option<usize>,
}

impl std::fmt::Debug for AryRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ArrayRef({}, {:?} {:?})", self.name, self.dim, self.base)
    }
}

/// Type alias for the iteration vector, with i32 elements.
pub type IterVec = Vec<i32>;
/// Type alias for the array access indices, with usize elements.
pub type AryAcc = Vec<usize>;

impl Node {
    /// Create a new Node with a given statement.
    pub fn new_node(a_stmt: Stmt) -> Rc<Node> {
        Rc::new(Node {
            stmt: a_stmt,
            parent: RefCell::new(Weak::new()),
        })
    }

    pub fn new_ref(
        ary_nm: &str,
        ary_dim: Vec<usize>,
        ary_sub: fn(&Vec<i32>) -> Vec<usize>,
    ) -> Rc<Node> {
        let ref_stmt = AryRef {
            name: ary_nm.to_string(),
            dim: ary_dim,
            sub: Box::new(ary_sub),
            base: None,
        };
        Node::new_node(Stmt::Ref(ref_stmt))
    }

    /// Create a new Node representing a simple loop with a fixed range.
    pub fn new_single_loop(ivar: &str, low: i32, high: i32) -> Rc<Node> {
        let loop_stmt = LoopStmt {
            iv: ivar.to_string(),
            lb: LoopBound::Fixed(low),
            ub: LoopBound::Fixed(high),
            // test: |i| i<ub , step: |k| k+1,
            test: Box::new(|i, ub| i < ub),
            step: Box::new(|i| i + 1),
            body: RefCell::new(vec![]),
        };
        Node::new_node(Stmt::Loop(loop_stmt))
    }

    /// Extend the body of a loop node with another node.
    pub fn extend_loop_body(loop_ref: &Rc<Node>, stmt_ref: &Rc<Node>) {
        if let Stmt::Loop(ref lp) = loop_ref.stmt {
            // officiating the parent-child relationship
            *stmt_ref.parent.borrow_mut() = Rc::downgrade(loop_ref);
            // adding to the body
            lp.body.borrow_mut().push(Rc::clone(stmt_ref));
        } else {
            panic!("extend_loop_body called on non-loop node");
        }
    }

    pub fn loop_only<U, F>(&self, f: F) -> Option<U>
    where
        F: FnOnce(&LoopStmt) -> U,
    {
        match &self.stmt {
            Stmt::Loop(ref aloop) => Some(f(aloop)),
            _ => None,
        }
    }

    pub fn ref_only<U, F>(&self, f: F) -> Option<U>
    where
        F: FnOnce(&AryRef) -> U,
    {
        match &self.stmt {
            Stmt::Ref(ref a_ref) => Some(f(a_ref)),
            _ => None,
        }
    }

    pub fn ref_only_ref<'a, U, F>(&'a self, f: F) -> Option<&'a U>
    where
        F: FnOnce(&'a AryRef) -> &'a U,
    {
        match &self.stmt {
            Stmt::Ref(ref a_ref) => Some(f(a_ref)),
            _ => None,
        }
    }

    pub fn ref_only_mut_ref<'a, U, F>(&'a mut self, f: F) -> Option<&'a mut U>
    where
        F: FnOnce(&'a mut AryRef) -> &'a mut U,
    {
        match &mut self.stmt {
            Stmt::Ref(ref mut a_ref) => Some(f(a_ref)),
            _ => None,
        }
    }

    // pub fn loop_body<'a>(&'a self, i: usize) -> &'a Rc<Node> {
    // }

    pub fn get_lb(&self) -> Option<i32> {
        self.loop_only(|lp| {
            if let LoopBound::Fixed(lowerbound) = lp.lb {
                lowerbound
            } else {
                panic!("dynamic loop bound is not supported")
            }
        })
    }

    pub fn get_ub(&self) -> Option<i32> {
        self.loop_only(|lp| {
            if let LoopBound::Fixed(upperbound) = lp.ub {
                upperbound
            } else {
                panic!("dynamic loop bound is not supported")
            }
        })
    }

    // Get the count of nodes in the loop tree.
    #[allow(dead_code)]
    pub fn node_count(&self) -> u32 {
        match &self.stmt {
            //    The body of a loop is a vector of Node's, so we need to
            //    iterate over the vector and sum the sanity of each node.
            Stmt::Loop(a_loop) => {
                1 + a_loop
                    .body
                    .borrow()
                    .iter()
                    .map(|x| x.as_ref().node_count())
                    .sum::<u32>()
            }
            Stmt::Ref(_) => 1,
            Stmt::Block(children) => {
                1 + children
                    .iter()
                    .map(|x| x.as_ref().node_count())
                    .sum::<u32>()
            }
        }
    }
}

// impl RefStmt {
//     fn my_nest(&self) -> Node {
//         // follow the parent pointers and return all enclosing loops as a Node
//         // let mut parent = self.parent.borrow().upgrade();
//         // while let Some(p) = parent {
//         //     match p {
//         //         Node::Loop(_) => return p,
//         //         Node::Ref(_) => parent = p.parent.borrow().upgrade(),
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
        let ar = AryRef {
            name: "X".to_string(),
            dim: vec![10],
            sub: Box::new(|iv| vec![(iv[0] as usize) + 1]),
            base: None,
        };
        assert_eq!((ar.sub)(&vec![1]), [2]);
    }

    #[test]
    fn matmul() {
        let n: usize = 100; // array dim
        let ubound = n as i32; // loop bound
                               // creating C[i,j] += A[i,k] * B[k,j]
        let s_ref_c = Node::new_ref("C", vec![n, n], |ijk| {
            vec![ijk[0] as usize, ijk[1] as usize]
        });
        let s_ref_a = Node::new_ref("A", vec![n, n], |ijk| {
            vec![ijk[0] as usize, ijk[2] as usize]
        });
        let s_ref_b = Node::new_ref("B", vec![n, n], |ijk| {
            vec![ijk[2] as usize, ijk[1] as usize]
        });

        // creating loop k = 0, n { s_ref }
        let k_loop_ref = Node::new_single_loop("k", 0, ubound);
        Node::extend_loop_body(&k_loop_ref, &s_ref_c);
        Node::extend_loop_body(&k_loop_ref, &s_ref_a);
        Node::extend_loop_body(&k_loop_ref, &s_ref_b);
        // creating loop j = 0, n
        let j_loop_ref = Node::new_single_loop("j", 0, ubound);
        Node::extend_loop_body(&j_loop_ref, &k_loop_ref);
        // creating loop i = 0, n
        let i_loop_ref = Node::new_single_loop("i", 0, ubound);
        Node::extend_loop_body(&i_loop_ref, &j_loop_ref);

        assert_eq!(i_loop_ref.node_count(), 6);
    }

    #[test]
    fn mat_transpose1() {
        let n: usize = 1024;
        let ubound = n as i32;
        // for (int c0 = 0; c0 < n; c0 += 1)
        // for (int c1 = 0; c1 < n; c1 += 1)
        //   x1[c0] = (x1[c0] + (A[c0][c1] * y_1[c1]));
        let s_ref_x1 = Node::new_ref("x1", vec![n], |ij| vec![ij[0] as usize]);
        let s_ref_a = Node::new_ref("a", vec![n, n], |ij| vec![ij[0] as usize, ij[1] as usize]);
        let s_ref_y1 = Node::new_ref("y1", vec![n], |ij| vec![ij[1] as usize]);

        let j_loop_ref = Node::new_single_loop("j", 0, ubound);
        Node::extend_loop_body(&j_loop_ref, &s_ref_x1);
        Node::extend_loop_body(&j_loop_ref, &s_ref_a);
        Node::extend_loop_body(&j_loop_ref, &s_ref_y1);

        let i_loop_ref = Node::new_single_loop("i", 0, ubound);
        Node::extend_loop_body(&i_loop_ref, &j_loop_ref);

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
    //     // let s_ref = Node::new_node(Stmt::Ref(s));

    //     let s_ref_x1 = Node::new_node(Stmt::Ref(x2));
    //     let s_ref_a = Node::new_node(Stmt::Ref(a));
    //     let s_ref_y1 = Node::new_node(Stmt::Ref(y2));

    //     let j_loop_ref = Node::new_single_loop("j", 0, ubound);
    //     Node::extend_loop_body(&j_loop_ref, &s_ref_x1);
    //     Node::extend_loop_body(&j_loop_ref, &s_ref_a);
    //     Node::extend_loop_body(&j_loop_ref, &s_ref_y1);

    //     let i_loop_ref = Node::new_single_loop("i", 0, ubound);
    //     Node::extend_loop_body(&i_loop_ref, &j_loop_ref);

    //     assert_eq!(i_loop_ref.node_count(), 5);
    // }
}
