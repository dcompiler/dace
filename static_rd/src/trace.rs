use dace::arybase::set_arybase;
use dace::ast::{AryRef, LoopBound, Node, Stmt};
use fxhash::FxHashMap;
use hist::Hist;
use list_serializable::ListSerializable;
use stack_alg_sim::stack::LRUStack;
use stack_alg_sim::vec::LRUVec;
use stack_alg_sim::LRU;
use std::collections::hash_map::Entry;

use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering};
use tracing::debug;

static COUNTER: AtomicI64 = AtomicI64::new(0);
const DS: usize = 8;
const CLS: usize = 64;

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

pub fn access3addr(ary_ref: &AryRef, ivec: &[i32]) -> usize {
    let ary_index = (ary_ref.sub)(ivec);
    if ary_index.len() != ary_ref.dim.len() {
        panic!("array index and dimension do not match");
    }

    let offset = ary_index
        .iter()
        .zip(ary_ref.dim.iter())
        .fold(0, |acc, (&i, &d)| acc * d + i);

    (ary_ref.base.unwrap() + offset) * DS / CLS
}

fn trace_rec(
    code: &Rc<Node>,
    ivec: &[i32],
    sim: &mut Box<dyn LRU<usize>>,
    hist: &mut Hist,
    data_accesses: &mut ListSerializable,
) {
    match &code.stmt {
        Stmt::Ref(ary_ref) => {
            let addr = access2addr(ary_ref, ivec);
            data_accesses.add(addr);
            let rd = sim.rec_access(addr);
            hist.add_dist(rd);
        }
        Stmt::Loop(aloop) => {
            if let LoopBound::Fixed(lb) = aloop.lb {
                if let LoopBound::Fixed(ub) = aloop.ub {
                    (lb..ub).for_each(|i| {
                        aloop.body.iter().for_each(|stmt| {
                            let mut myvec = ivec.to_owned();
                            myvec.push(i);
                            trace_rec(stmt, &myvec, sim, hist, data_accesses)
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
            .for_each(|s| trace_rec(s, ivec.clone(), sim, hist, data_accesses)),
        Stmt::Branch(stmt) => {
            if (stmt.cond)(ivec) {
                trace_rec(&stmt.then_body, ivec.clone(), sim, hist, data_accesses)
            } else if let Some(else_body) = &stmt.else_body {
                trace_rec(else_body, ivec.clone(), sim, hist, data_accesses)
            }
        }
    }
}

pub fn trace(code: &mut Rc<Node>, lru_type: &str, accesses_count: &mut ListSerializable) -> Hist {
    let mut sim: Box<dyn LRU<usize>> = if lru_type == "Vec" {
        Box::new(LRUVec::<usize>::new())
    } else {
        Box::new(LRUStack::<usize>::new())
    };

    let mut hist = Hist::new();
    set_arybase(code);
    println!("{:?}", code);
    trace_rec(
        code,
        &Vec::<i32>::new(),
        &mut sim,
        &mut hist,
        accesses_count,
    );
    hist
}

pub fn tracing_ri(code: &mut Rc<Node>) -> Hist {
    let mut hist = Hist::new();
    #[allow(non_snake_case)]
    let mut LAT_hash: FxHashMap<String, FxHashMap<u64, i64>> = Default::default();
    set_arybase(code);
    trace_ri(code, &mut LAT_hash, &[], &mut hist);
    hist
}

#[allow(non_snake_case)]
fn trace_ri(
    code: &Rc<Node>,
    LAT_hash: &mut FxHashMap<String, FxHashMap<u64, i64>>,
    ivec: &[i32],
    hist: &mut Hist,
) {
    match &code.stmt {
        Stmt::Ref(ary_ref) => {
            // debug!("trace_ri arr ref: {:#?}", code);
            let addr = access3addr(ary_ref, ivec) as u64;
            // debug!("addr: {}", addr);
            let str_name = ary_ref.name.clone();
            let mut prev_counter: Option<i64> = None;
            let local_counter = COUNTER.load(Ordering::Relaxed);
            match LAT_hash.entry(str_name) {
                Entry::Occupied(mut entry) => {
                    // *entry.entry(addr).and_.or_insert(0) += 1;
                    // entry.entry(addr).and_modify(|e| *e += 1).or_insert(counter);
                    match entry.get_mut().entry(addr) {
                        Entry::Occupied(mut inner) => {
                            prev_counter = Some(inner.insert(local_counter));
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(local_counter);
                        }
                    }
                }
                Entry::Vacant(entry) => {
                    let mut inner_hash: FxHashMap<u64, i64> = Default::default();
                    inner_hash.insert(addr, local_counter);
                    entry.insert(inner_hash);
                }
            }
            if local_counter == 8421375 {
                println!("LAT_hash: {:#?}", LAT_hash);
            }
            let mut ri: Option<_> = None;
            if let Some(prev_counter) = prev_counter {
                //update ri
                ri = Some((local_counter - prev_counter) as usize);
            } // FIXME: hist seems weird, how to deal with -1(the ri of never accessed again elements)
            hist.add_dist(ri);
            COUNTER.fetch_add(1, Ordering::Relaxed);
            debug!("counter: {}", COUNTER.load(Ordering::Relaxed));
            debug!("LAT_hash:{:#?}", LAT_hash);
            debug!("hist: {}", hist);
        }
        Stmt::Loop(aloop) => {
            // debug!("trace_ri loop ref: {:#?}", code);
            if let LoopBound::Fixed(lb) = aloop.lb {
                if let LoopBound::Fixed(ub) = aloop.ub {
                    (lb..ub).for_each(|i| {
                        aloop.body.iter().for_each(|stmt| {
                            let mut myvec = ivec.to_owned();
                            myvec.push(i);
                            trace_ri(stmt, LAT_hash, &myvec, hist)
                        })
                    })
                } else {
                    panic!("dynamic loop upper bound is not supported")
                }
            } else {
                panic!("dynamic loop lower bound is not supported")
            }
        }
        Stmt::Block(blk) => {
            // debug!("trace_ri block ref: {:#?}", code);
            blk.iter()
                .for_each(|s| trace_ri(s, LAT_hash, ivec.clone(), hist))
        }
        Stmt::Branch(stmt) => {
            if (stmt.cond)(ivec) {
                trace_ri(&stmt.then_body, LAT_hash, ivec, hist)
            } else if let Some(else_body) = &stmt.else_body {
                trace_ri(else_body, LAT_hash, ivec, hist)
            }
        }
    }
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

        let hist = trace(&mut aloop, "Stack", &mut ListSerializable::new());
        assert_eq!(hist.to_vec()[0], (None, 10));
        println!("{}", hist);
    }

    #[test]
    fn loop_a_0() {
        // i = 0, 10 { a[0] }
        let mut aref = Node::new_ref("A", vec![1], |_| vec![0]);
        let mut aloop = Node::new_single_loop("i", 0, 10);
        Node::extend_loop_body(&mut aloop, &mut aref);

        let hist = trace(&mut aloop, "Stack", &mut ListSerializable::new());
        assert_eq!(hist.to_vec()[0], (Some(1), 9));
        assert_eq!(hist.to_vec()[1], (None, 1));
        println!("{}", hist);
    }
}
