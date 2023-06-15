#![allow(dead_code)]
use dace::ast::Node;
use std::rc::Rc;

mod polybench;

pub fn matmul(n: usize) -> Rc<Node> {
    // n: usize is array dim
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

    i_loop_ref
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matmul_test() {
        let mm = matmul(100);
        assert_eq!(mm.node_count(), 6);
    }
}
