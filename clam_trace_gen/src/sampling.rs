use crate::ri_utils::access3addr;
use dace::arybase::set_arybase;
use dace::ast::{LoopBound, Node, Stmt};
use dace::iter::Walk;
use fxhash::FxHashMap;
use hist::Hist;
use std::collections::hash_map::Entry;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering};

static COUNTER: AtomicI64 = AtomicI64::new(0);

pub fn assign_ref_id(node: &mut Rc<Node>) {
    println!("Assigning ID...");
    let mut counter = 0;
    Walk::new(node)
        .filter(|node| matches!(&node.stmt, Stmt::Ref(_)))
        .for_each(|mut node| {
            let mutable = unsafe { Rc::get_mut_unchecked(&mut node) };
            let my_ref_id = mutable.ref_only_mut_ref(|aref| &mut aref.ref_id).unwrap();
            if my_ref_id.is_none() {
                *my_ref_id = Some(counter);
                counter += 1;
            }
        });
    println!("number of ID assigned: {}", counter);
}

pub fn print_tree(node: &Rc<Node>, level: usize) {
    print!("{:indent$}", "", indent = level * 2);
    match &node.stmt {
        Stmt::Ref(aref) => match aref.ref_id {
            Some(id) => println!("Ref(id: {}): {:?}", id, aref),
            None => println!("Ref(no id): {:?}", aref),
        },
        Stmt::Loop(aloop) => println!("Loop: {:?}", aloop),
        Stmt::Block(blk) => println!("Block: {:?}", blk),
        Stmt::Branch(stmt) => println!("Branch: {:?}", stmt),
    }

    match &node.stmt {
        Stmt::Loop(aloop) => {
            for child in &aloop.body {
                print_tree(child, level + 1);
            }
        }
        Stmt::Block(blk) => {
            for child in blk {
                print_tree(child, level + 1);
            }
        }
        Stmt::Branch(stmt) => {
            print_tree(&stmt.then_body, level + 1);
            if let Some(else_body) = &stmt.else_body {
                print_tree(else_body, level + 1);
            }
        }
        _ => {}
    }
}

pub fn tracing_ri(code: &mut Rc<Node>) -> Hist {
    let mut hist = Hist::new();
    let mut lat_hash: FxHashMap<String, FxHashMap<u64, i64>> = Default::default();
    let mut csv = String::new();
    csv.push_str("Label\tReuse Interval\tTag\tLogical Time\n");
    set_arybase(code);
    assign_ref_id(code);
    // print_tree(code, 0);
    println!("Tracing Reuse Interval...");
    trace_ri(code, &mut lat_hash, &[], &mut hist, &mut csv);

    println!("Writing to file...");
    let mut file = File::create("output.csv").expect("Unable to create file");
    file.write_all(csv.as_bytes())
        .expect("Unable to write data");

    let hist_data = hist.to_string();
    let mut hist_file = File::create("hist_output.txt").expect("Unable to create hist file");
    hist_file
        .write_all(hist_data.as_bytes())
        .expect("Unable to write hist data");

    hist
}

fn trace_ri(
    code: &Rc<Node>,
    lat_hash: &mut FxHashMap<String, FxHashMap<u64, i64>>,
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

            match lat_hash.entry(str_name) {
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
            let ref_label = ary_ref.ref_id.unwrap();
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
                            trace_ri(stmt, lat_hash, &myvec, hist, csv)
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
            .for_each(|s| trace_ri(s, lat_hash, ivec.clone(), hist, csv)),
        Stmt::Branch(stmt) => {
            if (stmt.cond)(ivec) {
                trace_ri(&stmt.then_body, lat_hash, ivec, hist, csv)
            } else if let Some(else_body) = &stmt.else_body {
                trace_ri(else_body, lat_hash, ivec, hist, csv)
            }
        }
    }
}
