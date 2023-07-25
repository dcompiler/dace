use crate::ri_utils::access3addr;
use dace::arybase::set_arybase;
use dace::ast::{LoopBound, Node, Stmt};
use fxhash::FxHashMap;
use hist::Hist;
use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering};
use tracing::debug;

static COUNTER: AtomicI64 = AtomicI64::new(0);
static REF_COUNTER: AtomicI64 = AtomicI64::new(0);

pub fn tracing_ri(code: &mut Rc<Node>) -> Hist {
    let mut hist = Hist::new();
    let mut lat_hash: FxHashMap<String, FxHashMap<u64, i64>> = Default::default();
    let mut ref_id_hash: FxHashMap<String, i64> = Default::default();
    let mut csv = String::new();
    csv.push_str("Label\tReuse Interval\tTag\tLogical Time\n");
    set_arybase(code);
    trace_ri(
        code,
        &mut lat_hash,
        &mut ref_id_hash,
        &[],
        &mut hist,
        &mut csv,
    );
    let mut file = File::create("output.csv").expect("Unable to create file");
    file.write_all(csv.as_bytes())
        .expect("Unable to write data");
    hist
}

fn trace_ri(
    code: &Rc<Node>,
    LAT_hash: &mut FxHashMap<String, FxHashMap<u64, i64>>,
    ref_id_hash: &mut FxHashMap<String, i64>,
    ivec: &[i32],
    hist: &mut Hist,
    csv: &mut String,
) {
    match &code.stmt {
        Stmt::Ref(ary_ref) => {
            let addr = access3addr(ary_ref, ivec) as u64;
            let str_name = ary_ref.name.clone();
            let mut prev_counter: Option<i64> = None;
            let local_counter = COUNTER.load(Ordering::Relaxed);

            let ref_id = match ref_id_hash.entry(str_name.clone()) {
                Entry::Occupied(entry) => *entry.get(),
                Entry::Vacant(entry) => {
                    let id = REF_COUNTER.fetch_add(1, Ordering::Relaxed);
                    entry.insert(id);
                    id
                }
            };

            match LAT_hash.entry(str_name) {
                Entry::Occupied(mut entry) => match entry.get_mut().entry(addr) {
                    Entry::Occupied(mut inner) => {
                        prev_counter = Some(inner.insert(local_counter));
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(local_counter);
                    }
                },
                Entry::Vacant(entry) => {
                    let mut inner_hash: FxHashMap<u64, i64> = Default::default();
                    inner_hash.insert(addr, local_counter);
                    entry.insert(inner_hash);
                }
            }
            let mut ri: Option<_> = None;
            if let Some(prev_counter) = prev_counter {
                ri = Some((local_counter - prev_counter) as usize);
            }
            hist.add_dist(ri);
            COUNTER.fetch_add(1, Ordering::Relaxed);
            let ref_label = ref_id.to_string();
            let reuse_interval = ri.map_or("-1".to_string(), |ri| ri.to_string());
            let addr_str = addr.to_string();
            let local_counter_str = local_counter.to_string();
            let line = format!(
                "{}\t{}\t{}\t{}\n",
                ref_label, reuse_interval, addr_str, local_counter_str
            );
            csv.push_str(&line);
        }
        Stmt::Loop(aloop) => {
            if let LoopBound::Fixed(lb) = aloop.lb {
                if let LoopBound::Fixed(ub) = aloop.ub {
                    (lb..ub).for_each(|i| {
                        aloop.body.iter().for_each(|stmt| {
                            let mut myvec = ivec.to_owned();
                            myvec.push(i);
                            trace_ri(stmt, LAT_hash, ref_id_hash, &myvec, hist, csv)
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
            .for_each(|s| trace_ri(s, LAT_hash, ref_id_hash, ivec.clone(), hist, csv)),
        Stmt::Branch(stmt) => {
            if (stmt.cond)(ivec) {
                trace_ri(&stmt.then_body, LAT_hash, ref_id_hash, ivec, hist, csv)
            } else if let Some(else_body) = &stmt.else_body {
                trace_ri(else_body, LAT_hash, ref_id_hash, ivec, hist, csv)
            }
        }
    }
}
