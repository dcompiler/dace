use crate::ast::*;
use std::iter::Iterator;
use std::rc::Rc;

pub struct Walk {
    // usize is the current body statement index, if there is any
    stack: Vec<(Rc<LoopTNode>, usize)>,
}

impl Walk {
    pub fn new(root: &Rc<LoopTNode>) -> Self {
        Walk {
            stack: vec![(root.clone(), 0)],
        }
        // Iter{ stack: vec![(root, root.loop_only( |lp| {
        //     if lp.body.borrow().len() > 0 { Some(0) } else { None } }))] }
    }

    fn step(&mut self) -> Option<Rc<LoopTNode>> {
        match self.stack.last().cloned() {
            None => None, // stack already empty
            Some((node, visited)) => {
                // if none has been visited, this is the first time we enter the node
                let mut result = None;
                if visited == 0 {
                    result = Some(node.clone());
                }
                match &node.as_ref().stmt {
                    Stmt::Loop(children) => {
                        if visited >= children.body.borrow().len() {
                            self.stack.pop();
                        } else {
                            self.stack.last_mut().unwrap().1 += 1;
                            self.stack
                                .push((children.body.borrow()[visited].clone(), 0));
                        }
                    }
                    Stmt::Ref(_) => {
                        self.stack.pop();
                    }
                    Stmt::Block(children) => {
                        if visited >= children.len() {
                            self.stack.pop();
                        } else {
                            self.stack.last_mut().unwrap().1 += 1;
                            self.stack.push((children[visited].clone(), 0));
                        }
                    }
                }
                result
            }
        }
    }
}

impl Iterator for Walk {
    type Item = Rc<LoopTNode>;
    fn next(&mut self) -> Option<Self::Item> {
        while !self.stack.is_empty() {
            if let Some(x) = self.step() {
                return Some(x);
            }
        }
        None
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

        let awalk = Walk::new(&aloop);
        assert_eq!(awalk.fold(0, |cnt, _stmt| cnt + 1), 2);
    }

    #[test]
    fn loop_ij() {
        // i = 0, 1, {j = 0, 0 n { a[0] }; b[0]
        let aref = LoopTNode::new_ref("A", vec![1], |_| vec![0]);
        let jloop = LoopTNode::new_single_loop("j", 0, 10);
        LoopTNode::extend_loop_body(&jloop, &aref);
        let bref = LoopTNode::new_ref("B", vec![1], |_| vec![0]);
        let iloop = LoopTNode::new_single_loop("i", 0, 1);
        LoopTNode::extend_loop_body(&iloop, &jloop);
        LoopTNode::extend_loop_body(&iloop, &bref);
        let awalk = Walk::new(&iloop);
        assert_eq!(awalk.fold(0, |cnt, _stmt| cnt + 1), 4);
    }
}
