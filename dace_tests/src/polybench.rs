#![allow(dead_code, non_snake_case)]
use dace::ast::Node;
use dace::ast::Stmt;
use dace::loop_node;
use std::rc::Rc;

fn trmm_trace(M: usize, N: usize) -> Rc<Node> {
    let i_loop_ref = Node::new_single_loop("i", 0, M as i32);
    let j_loop_ref = Node::new_single_loop("j", 0, N as i32);
    let k_loop_ref = Node::new_single_loop("k", Node::get_lb(&i_loop_ref).unwrap() + 1, M as i32);

    // B[i * N + j] += A[k * M + i] * B[k * N + j];
    let a_ref = Node::new_ref("A", vec![N, M], |ijk| {
        vec![ijk[2] as usize, ijk[0] as usize]
    });
    let b1_ref = Node::new_ref("B", vec![M, N], |ijk| {
        vec![ijk[2] as usize, ijk[1] as usize]
    });
    let b2_ref = Node::new_ref("B", vec![M, N], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    Node::extend_loop_body(&k_loop_ref, &a_ref);
    Node::extend_loop_body(&k_loop_ref, &b1_ref);
    Node::extend_loop_body(&k_loop_ref, &b2_ref);

    // B[i * N + j] = alpha * B[i * N + j];
    let b3_ref = Node::new_ref("B", vec![M, N], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    Node::extend_loop_body(&j_loop_ref, &b3_ref);
    Node::extend_loop_body(&j_loop_ref, &k_loop_ref);

    Node::extend_loop_body(&i_loop_ref, &j_loop_ref);

    i_loop_ref
}

pub fn mvt(n: usize) -> Rc<Node> {
    // n : usize is size of array
    let ubound = n as i32;

    // creating x1[i] = x1[i] + a[i][j] * y1[j];
    let s_ref_x1: Rc<Node> = Node::new_ref("x1", vec![n], |ij| vec![ij[0] as usize]);
    let s_ref_a1 = Node::new_ref("a1", vec![n, n], |ij| vec![ij[0] as usize, ij[1] as usize]);
    let s_ref_y1 = Node::new_ref("y1", vec![n], |ij| vec![ij[1] as usize]);

    // creating loop j = 0, n { s_ref }
    let j_loop_ref = Node::new_single_loop("j", 0, ubound);
    Node::extend_loop_body(&j_loop_ref, &s_ref_x1);
    Node::extend_loop_body(&j_loop_ref, &s_ref_a1);
    Node::extend_loop_body(&j_loop_ref, &s_ref_y1);
    Node::extend_loop_body(&j_loop_ref, &s_ref_x1);

    // creating loop i = 0, n
    let i_loop_ref = Node::new_single_loop("i", 0, ubound);
    Node::extend_loop_body(&i_loop_ref, &j_loop_ref);

    //x2[i] = x2[i] + a[j][i] * y2[j];
    let s_ref_x2: Rc<Node> = Node::new_ref("x2", vec![n], |ij| vec![ij[0] as usize]);
    let s_ref_a2 = Node::new_ref("a2", vec![n, n], |ij| vec![ij[1] as usize, ij[0] as usize]);
    let s_ref_y2 = Node::new_ref("y2", vec![n], |ij| vec![ij[1] as usize]);

    // creating loop k = 0, n { s_ref }
    let k_loop_ref = Node::new_single_loop("k", 0, ubound);
    Node::extend_loop_body(&k_loop_ref, &s_ref_x2);
    Node::extend_loop_body(&k_loop_ref, &s_ref_a2);
    Node::extend_loop_body(&k_loop_ref, &s_ref_y2);
    Node::extend_loop_body(&k_loop_ref, &s_ref_x2);

    // creating loop m = 0, n
    let m_loop_ref = Node::new_single_loop("m", 0, ubound);
    Node::extend_loop_body(&m_loop_ref, &k_loop_ref);

    // combine two seperate loops
    Node::new_node(Stmt::Block(vec![i_loop_ref, m_loop_ref]))
}

pub fn trisolv(n: usize) -> Rc<Node> {
    // n : usize is size of array
    let ubound = n as i32;

    // creating x[i] = b[i];
    let s_ref_x1 = Node::new_ref("x", vec![n], |ij| vec![ij[0] as usize]);
    let s_ref_b = Node::new_ref("b", vec![n], |ij| vec![ij[0] as usize]);

    // creating x[i] -= L[i][j] * x[j];
    let s_ref_L1 = Node::new_ref("L", vec![n, n], |ij| vec![ij[0] as usize, ij[1] as usize]);
    let s_ref_x2 = Node::new_ref("x", vec![n], |ij| vec![ij[1] as usize]);
    let s_ref_x3 = Node::new_ref("x", vec![n], |ij| vec![ij[0] as usize]);

    // creating x[i] = x[i] / L[i][i]
    let s_ref_L2 = Node::new_ref("L", vec![n, n], |ij| vec![ij[0] as usize, ij[0] as usize]);
    // s_ref_x1

    let j_loop_ref = Node::new_single_loop_dyn_ub("j", 0, move |i| i[0]);
    Node::extend_loop_body(&j_loop_ref, &s_ref_L1);
    Node::extend_loop_body(&j_loop_ref, &s_ref_x2);
    Node::extend_loop_body(&j_loop_ref, &s_ref_x3);
    Node::extend_loop_body(&j_loop_ref, &s_ref_x3);

    let i_loop_ref = Node::new_single_loop("i", 0, ubound);
    Node::extend_loop_body(&i_loop_ref, &s_ref_b);
    Node::extend_loop_body(&i_loop_ref, &s_ref_x1);
    Node::extend_loop_body(&i_loop_ref, &j_loop_ref);
    Node::extend_loop_body(&i_loop_ref, &s_ref_x1);
    Node::extend_loop_body(&i_loop_ref, &s_ref_L2);
    Node::extend_loop_body(&i_loop_ref, &s_ref_x1);

    i_loop_ref
}

pub fn syrk(n: usize, m: usize) -> Rc<Node> {
    // n,m are array dimensions
    let ubound1 = n as i32;
    let ubound2 = m as i32;

    //creating C[i][j] = C[i][j] * beta
    let s_ref_c1 = Node::new_ref("c", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    // creating C[i][j] = C[i][j] + alpha * A[i][k] * A[j][k]
    let s_ref_a1 = Node::new_ref("a1", vec![n, m], |ijk| {
        vec![ijk[0] as usize, ijk[2] as usize]
    });
    let s_ref_a2 = Node::new_ref("a2", vec![n, m], |ijk| {
        vec![ijk[1] as usize, ijk[2] as usize]
    });
    let s_ref_c2 = Node::new_ref("c", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    let j_loop_ref = Node::new_single_loop("j", 0, ubound1);
    Node::extend_loop_body(&j_loop_ref, &s_ref_c1);
    Node::extend_loop_body(&j_loop_ref, &s_ref_c1);

    let i_loop_ref = Node::new_single_loop("i", 0, ubound1);
    Node::extend_loop_body(&i_loop_ref, &j_loop_ref);

    let m_loop_ref = Node::new_single_loop("m", 0, ubound2);
    Node::extend_loop_body(&m_loop_ref, &s_ref_a1);
    Node::extend_loop_body(&m_loop_ref, &s_ref_a2);
    Node::extend_loop_body(&m_loop_ref, &s_ref_c2);
    Node::extend_loop_body(&m_loop_ref, &s_ref_c2);

    let l_loop_ref = Node::new_single_loop("l", 0, ubound1);
    Node::extend_loop_body(&l_loop_ref, &m_loop_ref);

    let k_loop_ref = Node::new_single_loop("k", 0, ubound1);
    Node::extend_loop_body(&k_loop_ref, &l_loop_ref);

    // combine two seperate loops
    Node::new_node(Stmt::Block(vec![i_loop_ref, k_loop_ref]))
}

pub fn syr2d(n: usize, m: usize) -> Rc<Node> {
    // n,m are array dimensions
    let ubound1 = n as i32;
    let ubound2 = m as i32;

    // creating C[i][j] *= beta;
    let s_ref_c = Node::new_ref("c", vec![n, n], |ij| vec![ij[0] as usize, ij[1] as usize]);

    // creating C[i][j] += A[j][k]*alpha*B[i][k] + B[j][k]*alpha*A[i][k];
    let s_ref_a1 = Node::new_ref("a1", vec![n, m], |ijkl| {
        vec![ijkl[3] as usize, ijkl[2] as usize]
    });
    let s_ref_b1 = Node::new_ref("b1", vec![n, m], |ijkl| {
        vec![ijkl[0] as usize, ijkl[2] as usize]
    });
    let s_ref_b2 = Node::new_ref("b2", vec![n, m], |ijkl| {
        vec![ijkl[3] as usize, ijkl[2] as usize]
    });
    let s_ref_a2 = Node::new_ref("a2", vec![n, m], |ijkl| {
        vec![ijkl[0] as usize, ijkl[2] as usize]
    });
    let s_ref_c1 = Node::new_ref("c1", vec![n, n], |ijkl| {
        vec![ijkl[0] as usize, ijkl[3] as usize]
    });
    let s_ref_c2 = Node::new_ref("c2", vec![n, n], |ijkl| {
        vec![ijkl[0] as usize, ijkl[3] as usize]
    });

    let l_loop_ref = loop_node!("l", 0 => |i : &[i32]| i[0]);
    Node::extend_loop_body(&l_loop_ref, &s_ref_a1);
    Node::extend_loop_body(&l_loop_ref, &s_ref_b1);
    Node::extend_loop_body(&l_loop_ref, &s_ref_b2);
    Node::extend_loop_body(&l_loop_ref, &s_ref_a2);
    Node::extend_loop_body(&l_loop_ref, &s_ref_c1);
    Node::extend_loop_body(&l_loop_ref, &s_ref_c2);

    let k_loop_ref = Node::new_single_loop("k", 0, ubound2);
    Node::extend_loop_body(&k_loop_ref, &l_loop_ref);

    let j_loop_ref = loop_node!("j", 0 => |i : &[i32]| i[0]);
    Node::extend_loop_body(&j_loop_ref, &s_ref_c);
    Node::extend_loop_body(&j_loop_ref, &s_ref_c);

    let i_loop_ref = Node::new_single_loop("i", 0, ubound1);
    Node::extend_loop_body(&i_loop_ref, &j_loop_ref);
    Node::extend_loop_body(&i_loop_ref, &k_loop_ref);

    i_loop_ref
}

fn _2mm(NI: usize, NJ: usize, NK: usize, NL: usize) -> Rc<Node> {
    let s_ref_tmp = Node::new_ref("tmp", vec![NI, NJ], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    let s_ref_a = Node::new_ref("a", vec![NI, NK], |ijk| {
        vec![ijk[0] as usize, ijk[2] as usize]
    });
    let s_ref_b = Node::new_ref("b", vec![NK, NJ], |ijk| {
        vec![ijk[2] as usize, ijk[1] as usize]
    });
    let s_ref_c = Node::new_ref("c", vec![NL, NJ], |ijk| {
        vec![ijk[3] as usize, ijk[1] as usize]
    });
    let s_ref_d = Node::new_ref("d", vec![NI, NL], |ijk| {
        vec![ijk[0] as usize, ijk[3] as usize]
    });

    let knk_loop_ref = Node::new_single_loop("k", 0, NK as i32);
    Node::extend_loop_body(&knk_loop_ref, &s_ref_a);
    Node::extend_loop_body(&knk_loop_ref, &s_ref_b);
    Node::extend_loop_body(&knk_loop_ref, &s_ref_tmp);
    
    let jnj_loop_ref = Node::new_single_loop("j", 0, NJ as i32);
    Node::extend_loop_body(&knk_loop_ref, &s_ref_tmp);
    Node::extend_loop_body(&knk_loop_ref, &knk_loop_ref);
    
    let ini_loop_ref1 = Node::new_single_loop("i", 0, NI as i32);
    Node::extend_loop_body(&ini_loop_ref1, &jnj_loop_ref);

    let knj_loop_ref = Node::new_single_loop("k", 0, NJ as i32);
    Node::extend_loop_body(&knj_loop_ref, &s_ref_tmp);
    Node::extend_loop_body(&knj_loop_ref, &s_ref_c);
    Node::extend_loop_body(&knj_loop_ref, &s_ref_d);
    
    let jnl_loop_ref = Node::new_single_loop("j", 0, NL as i32);
    Node::extend_loop_body(&jnj_loop_ref, &s_ref_d);
    Node::extend_loop_body(&jnj_loop_ref, &knj_loop_ref);

    let ini_loop_ref2 = Node::new_single_loop("i", 0, NI as i32);
    Node::extend_loop_body(&ini_loop_ref2, &jnl_loop_ref);


    Node::new_node(Stmt::Block(vec![ini_loop_ref1, ini_loop_ref2]))

}


#[cfg(test)]
mod tests {
    use super::*;
    fn trmm_trace_test() {
        let M = 1024;
        let N = 1024;

        let ast = trmm_trace(M, N);
        assert_eq!(ast.node_count(), 7);
    }

    #[test]
    fn test_mvt() {
        assert_eq!(mvt(1024).node_count(), 13);
    }

    #[test]
    fn test_trisolv() {
        assert_eq!(trisolv(1024).node_count(), 11);
    }

    #[test]
    fn test_syrk() {
        assert_eq!(syrk(256, 256).node_count(), 12);
    }

    #[test]
    fn test_syr2d() {
        assert_eq!(syr2d(1024, 1024).node_count(), 12);
        
    }
    #[test]
    fn _2mm_test() {
        assert_eq!(_2mm(1024, 1024, 1024, 1024).node_count(), 10);
    }
}
