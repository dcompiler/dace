use crate::loop_tree::*;
use crate::tree_walk::Walk;
use std::rc::Rc;
use std::collections::HashMap;

pub trait AryBase {
    fn set_arybase(&mut self);
}

impl AryBase for Rc<LoopTNode> {
    fn set_arybase(&mut self) {
	Walk::new(self).filter(|node| matches!(&node.stmt, Stmt::Ref(_)))
	    .fold( (HashMap::<&str, usize>::new(),0),
		    |(tbl, cur_base), node| {
			let new_ary = false;
			let ary_base = tbl.entry(&node.ref_only(|a_ref| a_ref.name))
			    .or_insert_with(|| {new_ary = true; cur_base} );
			if let Stmt::Ref(aref) = &node.stmt {
			    aref.base = Some(ary_base);
			}
			if new_ary {
			    let dim = &node.ref_only(|a_ref| a_ref.dim);
			    let new_size = dim.iter().reduce(|tot, d| tot*d);
			    cur_base += new_size;
			}
			(tbl, cur_base)} )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn nobase() {
	let node = LoopTNode::new_ref("A", vec![1], |_| vec![0]);
	if let Stmt::Ref(aref) = &node.stmt {
	    let b = aref.base.unwrap();
	}
    }

    fn matmul() {
    }
}
