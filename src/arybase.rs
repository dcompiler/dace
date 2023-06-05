use crate::loop_tree::*;
use std::rc::Rc;
use std::collections::HashMap;

pub trait AryBase { }

impl AryBase for Rc<LoopTNode> {
    pub fn set_arybase(&mut self) {
	self.walk().filter(|node| matches!(&node.stmt, Stmt::Ref(_)))
	    .fold( (HashMap::<&str, usize>::new(),0),
		    |(tbl, cur_base), aref| {
			let new_ary = false;
			let ary_base = tbl.entry(&aref.ary_only().name)
			    .or_insert_with(|| {new_ary = true; cur_base} );
			aref.ary_only().base = Some(ary_base);
			if (new_ary) {
			    let new_size = &aref.ary_only().dim.iter()
				.reduce(|tot, d| tot*d);
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
