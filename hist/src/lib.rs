use serde::{Deserialize, Serialize};
// use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;

//use csv::WriterBuilder;

#[derive(Serialize, Deserialize)]
pub struct Hist {
    hist: HashMap<Option<usize>, usize>,
    // attrs: HashMap<String,String>
}

impl Default for Hist {
    fn default() -> Self {
        Self::new()
    }
}

impl Hist {
    pub fn new() -> Hist {
        Hist {
            hist: HashMap::new(),
        }
    }

    pub fn add_dist(&mut self, d: Option<usize>) {
        self.hist
            .entry(d)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
    }

    pub fn to_vec(&self) -> Vec<(Option<usize>, usize)> {
        let mut h2 = self.hist.clone();
        let inf_rds = h2.remove(&None);
        let mut hvec: Vec<(Option<usize>, usize)> = h2.iter().map(|(x, y)| (*x, *y)).collect();
        hvec.sort_by(|a, b| a.0.cmp(&b.0));
        if let Some(cnt) = inf_rds {
            hvec.push((None, cnt));
        }
        hvec
    }
}

impl fmt::Display for Hist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut hvec = self.to_vec();
        let tot = hvec.iter().fold(0, |acc, x| acc + x.1);

        writeln!(
            f,
            "Reuse distance histogram: \n\t{} distance value(s), min {:?}, max {:?}\n\t{} accesses",
            hvec.len(),
            hvec[0].0,
            hvec[hvec.len() - 1].0,
            tot
        )?;
        if hvec[hvec.len() - 1].0.is_none() {
            writeln!(f, "\t({} first accesses)", hvec[hvec.len() - 1].1)?;
            hvec.pop();
        }
        writeln!(f, "value, count")?;
        hvec.into_iter()
            .fold(Ok(()), |_, (d, cnt)| writeln!(f, "{}, {}", d.unwrap(), cnt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut h = Hist::new();
        h.add_dist(None);
        h.add_dist(Some(1));
        h.add_dist(Some(1));
        h.add_dist(Some(100));

        let v = h.to_vec();
        assert_eq!(v[0], (Some(1), 2));
        assert_eq!(v[1], (Some(100), 1));
        assert_eq!(v[2], (None, 1));

        assert_eq!(
            format!("{}", h),
            "Reuse distance histogram: 
	3 distance value(s), min Some(1), max None
	4 accesses
	(1 first accesses)
value, count
1, 2
100, 1
"
        );

        // use cargo test -- --show-output to see the result
        println!("{}", h);
    }
}
