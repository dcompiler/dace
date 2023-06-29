#[derive(Debug)]
pub struct LRUVec<T> {
    pub stack: Vec<Option<Box<T>>>,
}

impl<T: PartialEq> Default for LRUVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PartialEq + Clone> crate::LRU<T> for LRUVec<T> {
    fn rec_access(&mut self, val: T) -> Option<usize> {
        self.rec_access_impl(val)
    }
}

impl<T: PartialEq> LRUVec<T> {
    pub fn new() -> LRUVec<T> {
        LRUVec {
            stack: Vec::<Option<Box<T>>>::new(),
        }
    }

    pub fn rec_access_impl(&mut self, val: T) -> Option<usize> {
        if self.stack.is_empty() {
            self.stack.push(Some(Box::new(val)));
            return None;
        }

        if **self.stack[0].as_ref().unwrap() == val {
            return Some(1);
        }

        let mut last = self.stack[0].take();
        for pos in 1..self.stack.len() {
            let temp = self.stack[pos].take();
            self.stack[pos] = last;
            last = temp;
            if **last.as_ref().unwrap() == val {
                self.stack[0] = last;
                return Some(pos + 1);
            }
        }
        // a cold miss
        self.stack.push(last); // add to the end of the vector
        self.stack[0] = Some(Box::new(val));
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LRU;

    #[test]
    fn cyclic() {
        let mut analyzer = LRUVec::<String>::new();
        let mut dists = Vec::new();
        // let st = "abc abc";
        for c in "abc abc".chars().filter(|c| !c.is_whitespace()) {
            dists.push(analyzer.rec_access(c.to_string()));
        }

        assert_eq!(dists, [None, None, None, Some(3), Some(3), Some(3)]);
    }

    #[test]
    fn cyclic_slice() {
        let mut analyzer = LRUVec::<&str>::new();
        let mut dists = Vec::new();
        let st = "abcabc";
        for i in 0..st.len() {
            dists.push(analyzer.rec_access(&st[i..i + 1]));
        }

        assert_eq!(dists, [None, None, None, Some(3), Some(3), Some(3)]);
    }

    #[test]
    fn sawtooth() {
        let mut analyzer = LRUVec::<String>::new();
        let mut dists = Vec::new();
        for c in "abc cba".chars().filter(|c| !c.is_whitespace()) {
            dists.push(analyzer.rec_access(c.to_string()));
        }

        assert_eq!(dists, [None, None, None, Some(1), Some(2), Some(3)]);
    }
}
