use crate::looptnode::*;
use lru_stack::LRUStack;
use hist::Hist;
use ascii_converter::string_to_decimals;
use std::rc::Rc;

// Handling a single array ONLY.
fn access2addr(ary_ref: &AryRef, ivec: &Vec<usize>) -> usize {
    // let ary_initial: usize = (string_to_decimals(&ary_ref.name).unwrap())[0] as usize;
    // let base = ary_initial * MAX_ARRAY;
    
    let ary_index = (ary_ref.sub)(ivec);
    if ary_index.len() != ary_ref.dim.len() { panic!("array index and dimension do not match"); }
    
    let offset = ary_index.iter().zip(ary_ref.dim.iter())
	.fold(0, |acc, (&i, &d)| acc*d + i);
    
    return offset;
}

fn trace_rec(code: &Rc<LoopTNode>, ivec: &Vec<usize>, sim: &mut LRUStack<usize>, hist: &mut Hist) {
    match &code.stmt {
	Stmt::Ref(ary_ref) => {let addr = access2addr(&ary_ref, &ivec);
			       let rd = sim.rec_access(addr);
			       hist.add_dist(rd);},
	Stmt::Loop(aloop) => {
	    let mut myvec = ivec.clone();
	    myvec.push(0);
	    if let LoopBound::Fixed(lb) = aloop.lb {
		if let LoopBound::Fixed(ub) = aloop.ub {
		    ((lb as usize)..(ub as usize)).into_iter().for_each(
			|i| {
			*myvec.last_mut().unwrap() = i;
			aloop.body.borrow().iter().for_each(
			    |stmt|
			    trace_rec(stmt, &myvec, sim, hist) )})
		}
		else {panic!("dynamic loop upper bound is not supported")}
	    }
	    else {panic!("dynamic loop lower bound is not supported")}
	}
    }
}

pub fn trace(code: &Rc<LoopTNode>) -> Hist {
    let mut sim = LRUStack::<usize>::new();
    let mut hist = Hist::new();
    trace_rec(code, &Vec::<usize>::new(), &mut sim, &mut hist);
    hist
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_access2addr() {
	let aij_node = LoopTNode::new_ref("x", vec![10,10], |ij| vec![ij[0], ij[1]]);
	if let Stmt::Ref(aij) = &aij_node.stmt {
	    assert_eq!(access2addr(&aij, &vec![0,0]), 0);
	    assert_eq!(access2addr(&aij, &vec![9,9]), 99);
	}
    }

    #[test]
    fn loop_a_i() {

        // i = 0, n { a[i] }
        let aref = LoopTNode::new_ref("A", vec![10], |i| vec![i[0]]);
        let aloop = LoopTNode::new_single_loop("i", 0, 10);
        LoopTNode::extend_loop_body(&aloop, &aref);

	let hist = trace(&aloop);
	assert_eq!(hist.to_vec()[0], (None, 10));
	println!("{}", hist);	
    }

    #[test]
    fn loop_a_0() {

        // i = 0, n { a[0] }
        let aref = LoopTNode::new_ref("A", vec![1], |_| vec![0]);
        let aloop = LoopTNode::new_single_loop("i", 0, 10);
        LoopTNode::extend_loop_body(&aloop, &aref);

	let hist = trace(&aloop);
	assert_eq!(hist.to_vec()[0], (Some(1), 9));
	assert_eq!(hist.to_vec()[1], (None, 1));
	println!("{}", hist);	
    }
}


// fro each array ref, calute adreaa
// for all the create, we can just add the call.


// In the unite test, add loop for one reference and check correctness.

//  How do we do sampling once we have trace?  ----> sampler.rs.



// For each reference, we get a loop node.
//
// for each sample, we get to the refer for that loop interation,
