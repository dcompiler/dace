use crate::ast::{LoopBound, LoopStmt, Node, Stmt};
use crate::iter::Walk;

use rand::prelude::*;
use std::collections::{BTreeSet, HashMap};
use std::intrinsics::ceilf32;
use std::ops::Range;
use std::rc::Rc;
use tracing::debug;

pub fn set_arybase(aloop: &mut Rc<Node>) -> (HashMap<String, usize>, usize) {
    let init = (HashMap::<String, usize>::new(), 0);
    Walk::new(aloop)
        .filter(|node| matches!(&node.stmt, Stmt::Ref(_)))
        .fold::<(HashMap<String, usize>, usize), _>(init, |(mut tbl, mut cur_base), mut node| {
            let ary_name = node.ref_only_ref(|a_ref| &a_ref.name).unwrap().as_str();
            if !tbl.contains_key(ary_name) {
                tbl.insert(ary_name.to_string(), cur_base);
                let dim = node.ref_only_ref(|a_ref| &a_ref.dim).unwrap();
                let ary_size: usize = dim.iter().product();
                cur_base += ary_size;
            }
            let ary_base = tbl.get(ary_name).unwrap();
            let mutable = unsafe { Rc::get_mut_unchecked(&mut node) };
            let my_base = mutable.ref_only_mut_ref(|a_ref| &mut a_ref.base).unwrap();
            *my_base = Some(*ary_base);
            (tbl, cur_base)
        })
}

pub fn sample_collect<'a>(
    code_tree: &'a Node,
    wrapping_loops: &mut Vec<&'a LoopStmt>,
    ans: &mut HashMap<usize, Vec<(&'a str, Range<usize>)>>,
    // access_name, (loop_name + sample_times)* + -
    ref_counter: &mut usize,
) {
    // let init = (HashMap::<String, usize>::new(), 0);
    match &code_tree.stmt {
        Stmt::Loop(stmt) => {
            wrapping_loops.push(stmt);
            for i in stmt.body.iter() {
                sample_collect(i, wrapping_loops, ans, ref_counter);
            }
            wrapping_loops.pop();
        }
        Stmt::Ref(_) => {
            let accesses: Vec<_> = wrapping_loops
                .iter()
                .filter_map(|x| {
                    let LoopBound::Fixed(lb) = x.lb else {
                        return None;
                    };
                    let LoopBound::Fixed(ub) = x.ub else {
                        return None;
                    };
                    Some((x.iv.as_str(), lb as usize..ub as usize))
                })
                .collect();
            // let ary_name = x.name;
            // we could use this to provide more information...
            ans.insert(*ref_counter, accesses);
            *ref_counter += 1;
        }
        Stmt::Block(x) => {
            for i in x.iter() {
                sample_collect(i, wrapping_loops, ans, ref_counter);
            }
        }
        Stmt::Branch(_) => unimplemented!("branch is not supported yet"),
    }
}

pub fn sample_gen(
    collected: &mut HashMap<usize, Vec<(&str, Range<usize>)>>,
    sampling_rate: f32,
) -> HashMap<usize, BTreeSet<Vec<usize>>> {
    let mut intermidiate = HashMap::<usize, f32>::new();
    collected.iter_mut().for_each(|(ref_id, accesses)| {
        let mut sample_times = 1.;
        accesses.iter_mut().for_each(|(_, range)| {
            let (lb, ub) = (range.start, range.end);
            sample_times *= (ub - lb) as f32 * sampling_rate;
        });
        intermidiate.insert(*ref_id, sample_times);
    });
    debug!("collected: {:#?}", collected);
    debug!("intermidiate: {:#?}", intermidiate);

    let mut ans = HashMap::<usize, BTreeSet<Vec<usize>>>::new();
    collected.iter_mut().for_each(|(ref_id, accesses)| {
        while ans.get(ref_id).unwrap_or(&BTreeSet::new()).len()
            < unsafe { ceilf32(*intermidiate.get(ref_id).unwrap()) as usize }
        // FIXME: no sure if f32 is enough here
        {
            let mut sample_name: Vec<usize> = Vec::new();
            accesses.iter_mut().for_each(|(_, range)| {
                let (lb, ub) = (range.start, range.end);
                let mut rng = rand::thread_rng();
                let dist = rand::distributions::Uniform::new(0, ub - lb);
                let rand_num = dist.sample(&mut rng);
                sample_name.push(rand_num);
            });
            ans.entry(*ref_id)
                .or_insert(BTreeSet::new())
                .insert(sample_name);
            // kinda weird here, feels like wasting a cycle of loop
        }
    });
    // ans.clone().into_iter().for_each(|(ref_id, samples)| {
    //     println!("ref_id: {}", ref_id);
    //     println!("samples: {}", samples.len());
    // });
    print!("ans: {:#?}", ans);
    ans
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn nobase() {
        let node = Node::new_ref("A", vec![1], |_| vec![0]);
        if let Stmt::Ref(aref) = &node.stmt {
            let _ = aref.base.unwrap();
        }
    }

    #[test]
    fn ary3() {
        let n: usize = 100; // array dim
        let ubound = n as i32; // loop bound
                               // creating A[i] B[i,i+1] C[i,i+1,i+2]
        let ref_a = Node::new_ref("A", vec![n], |i| vec![i[0] as usize]);
        let ref_b = Node::new_ref("B", vec![n, n], |i| vec![i[0] as usize, i[0] as usize + 1]);
        let ref_c = Node::new_ref("C", vec![n, n, n], |i| {
            vec![i[0] as usize, i[0] as usize + 1, i[0] as usize + 2]
        });

        // creating loop k = 0, n
        let mut iloop = Node::new_single_loop("i", 0, ubound);
        [ref_a, ref_b, ref_c]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(&mut iloop, s));

        let (tbl, size) = set_arybase(&mut iloop);
        assert_eq!(tbl.len(), 3);
        // println!("{:?}", tbl);
        assert_eq!(size, n + n * n + n * n * n);

        assert_eq!(
            iloop
                .loop_only(|lp| lp.body[0].ref_only(|rf| rf.base).unwrap())
                .unwrap(),
            Some(0)
        );
        assert_eq!(
            iloop
                .loop_only(|lp| lp.body[1].ref_only(|rf| rf.base).unwrap())
                .unwrap(),
            Some(n)
        );
        assert_eq!(
            iloop
                .loop_only(|lp| lp.body[2].ref_only(|rf| rf.base).unwrap())
                .unwrap(),
            Some(n + n * n)
        );
        // Walk::new(&iloop).for_each( |node| println!("{:?}", node) );
    }
}
