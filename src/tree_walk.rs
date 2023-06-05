use crate::loop_tree::*;
use std::rc::Rc;
use std::iter::Iterator;

pub struct Walk
{
    // usize is the current body statement index, if there is any
    stack: Vec<(Rc<LoopTNode>, Option<usize>)>,
}

impl Walk {
    pub fn new(root: & Rc<LoopTNode>) -> Walk {
	Walk{ stack: vec![(root.clone(), None)] }
	// Iter{ stack: vec![(root, root.loop_only( |lp| {
	//     if lp.body.borrow().len() > 0 { Some(0) } else { None } }))] }
    }
}

impl Iterator for Walk {
    type Item = Rc<LoopTNode>;

    fn next(&mut self) -> Option<Self::Item> {
	if self.stack.len() == 0 { return None }

	let (top_node, cur_i) = self.stack.pop().unwrap();
	match &top_node.stmt {
	    Stmt::Ref(_) => (), // dropping the Rc ref
	    Stmt::Loop(_) => {
		self.stack.push((top_node.clone(), cur_i));   // setting up the loop
		// This may take a while
		loop {
		    let (cur_top, cur_i) = self.stack.pop().unwrap();
		    let next_i = cur_i.map_or(0, |i| i+1);
		    let body_len = cur_top.loop_only(| s | s.body.borrow().len()).unwrap();
		    if next_i < body_len {
			self.stack.push( (cur_top.clone(), Some(next_i)) );
			if let Stmt::Loop(ref cur_loop) = cur_top.stmt {
			    let next_body_node = &cur_loop.body.borrow()[next_i];
			    self.stack.push( (next_body_node.clone(), None) );
			}
			break;
		    }
		    if self.stack.len() == 0 { return None }
		}	
	    }
	}
	Some(top_node)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn loop_a_0() {
        // i = 0, n { a[0] }
        let aref = LoopTNode::new_ref("A", vec![1], |_| vec![0]);
        let aloop = LoopTNode::new_single_loop("i", 0, 10);
        LoopTNode::extend_loop_body(&aloop, &aref);

	let awalk = Walk::new( &aloop );
	assert_eq!(awalk.fold(0, |cnt, stmt| cnt + 1 ), 2);
    }

    #[test]
    fn loop_ij() {
        // i = 0, 1, {j = 0, 0 n { a[0] }; b[0]
        let aref = LoopTNode::new_ref("A", vec![1], |_| vec![0]);
        let jloop = LoopTNode::new_single_loop("i", 0, 10);
        LoopTNode::extend_loop_body(&jloop, &aref);
        let bref = LoopTNode::new_ref("B", vec![1], |_| vec![0]);
	let iloop = LoopTNode::new_single_loop("j", 0, 1);
	LoopTNode::extend_loop_body(&iloop, &jloop);
	LoopTNode::extend_loop_body(&iloop, &bref);

	let awalk = Walk::new( &jloop );
	assert_eq!(awalk.fold(0, |cnt, stmt| cnt + 1 ), 4);
    }
}
	    
