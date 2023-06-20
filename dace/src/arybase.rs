use crate::ast::{Node, Stmt};
use crate::iter::Walk;

use std::collections::HashMap;
use std::rc::Rc;

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
        vec![ref_a, ref_b, ref_c]
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
