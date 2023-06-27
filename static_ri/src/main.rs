mod ri_utils;
mod sampling;
use dace::arybase;
use dace_tests::polybench;
use std::{collections::HashMap, time::Instant};
use tracing_subscriber::{prelude::*, EnvFilter};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("LOG_LEVEL"))
        .init();
    let mut wrapping_loop = Vec::new();
    let trace = polybench::gemm(128);
    let mut ref_coutner = 0;
    let start = Instant::now();
    // let hist = static_rd::trace::trace(&mut trace);
    // let hist = static_rd::trace::tracing_ri(&mut trace);
    let mut ans = HashMap::new();
    arybase::sample_collect(&trace, &mut wrapping_loop, &mut ans, &mut ref_coutner);
    let _samples: HashMap<usize, std::collections::BTreeSet<Vec<usize>>> =
        arybase::sample_gen(&mut ans, 0.1);
    let end = Instant::now();
    println!("gemm trace time: {:?}", end - start);
    // println!("collected: {ans:#?}");
    //println!("hist: {}", hist);
}
