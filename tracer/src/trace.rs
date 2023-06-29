use dace::arybase::set_arybase;
use dace::ast::{AryRef, LoopBound, Node, Stmt};
use list_serializable::ListSerializable;
use std::rc::Rc;
use hist::Hist;
use crate::calculate;
type Reuse = ListSerializable<(usize, Option<usize>)>;

fn access2addr(ary_ref: &AryRef, ivec: &[i32]) -> usize {
    let ary_index = (ary_ref.sub)(ivec);
    if ary_index.len() != ary_ref.dim.len() {
        panic!("array index and dimension do not match");
    }

    let offset = ary_index
        .iter()
        .zip(ary_ref.dim.iter())
        .fold(0, |acc, (&i, &d)| acc * d + i);

    ary_ref.base.unwrap() + offset
}

fn trace_rec(
    code: &Rc<Node>,
    ivec: &[i32],
    data_accesses: &mut ListSerializable<usize>,
) {
    match &code.stmt {
        Stmt::Ref(ary_ref) => {
            let addr = access2addr(ary_ref, ivec);
            data_accesses.add(addr);
        }
        Stmt::Loop(aloop) => {
            if let LoopBound::Fixed(lb) = aloop.lb {
                if let LoopBound::Fixed(ub) = aloop.ub {
                    (lb..ub).for_each(|i| {
                        aloop.body.iter().for_each(|stmt| {
                            let mut myvec = ivec.to_owned();
                            myvec.push(i);
                            trace_rec(stmt, &myvec, data_accesses)
                        })
                    })
                } else {
                    panic!("dynamic loop upper bound is not supported")
                }
            } else {
                panic!("dynamic loop lower bound is not supported")
            }
        }
        Stmt::Block(blk) => blk
            .iter()
            .for_each(|s| trace_rec(s, ivec.clone(), data_accesses)),
    }
}

pub fn trace(code: &mut Rc<Node>, lru_type: &str) -> (Hist, Hist, Reuse, Reuse, ListSerializable<usize>) {
    let mut accesses_count: ListSerializable<usize> = ListSerializable::<usize>::new();

    set_arybase(code);
    println!("{:?}", code);
    trace_rec(
        code,
        &Vec::<i32>::new(),
        &mut accesses_count,
    );

    let result = calculate::calculate_trace(&accesses_count, lru_type);

    (result.0, result.1, result.2, result.3, accesses_count)
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_access2addr() {
        let mut aij_node =
            Node::new_ref("x", vec![10, 10], |ij| vec![ij[0] as usize, ij[1] as usize]);
        let mutable = unsafe { Rc::get_mut_unchecked(&mut aij_node) };
        *mutable.ref_only_mut_ref(|a| &mut a.base).unwrap() = Some(0);
        if let Stmt::Ref(aij) = &aij_node.stmt {
            assert_eq!(access2addr(aij, &[0, 0]), 0);
            assert_eq!(access2addr(aij, &[9, 9]), 99);
        }
    }

    #[test]
    fn loop_a_i() {
        // i = 0, 10 { a[i] }
        let mut aref = Node::new_ref("A", vec![10], |i| vec![i[0] as usize]);
        let mut aloop = Node::new_single_loop("i", 0, 10);
        Node::extend_loop_body(&mut aloop, &mut aref);

        let hist = trace(&mut aloop, "Olken");
        assert_eq!(hist.0.to_vec()[0], (None, 10));
        println!("{}", hist.0);
    }

    #[test]
    fn loop_a_0() {
        // i = 0, 10 { a[0] }
        let mut aref = Node::new_ref("A", vec![1], |_| vec![0]);
        let mut aloop = Node::new_single_loop("i", 0, 10);
        Node::extend_loop_body(&mut aloop, &mut aref);

        let hist = trace(&mut aloop, "Olken");

        

        // assert_eq!(hist.0.to_vec()[0], (Some(1), 9));
        // assert_eq!(hist.0.to_vec()[1], (None, 1));
        println!("{}", hist.0);
    }
}
