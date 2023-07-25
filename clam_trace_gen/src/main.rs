mod ri_utils;
mod sampling;
use dace_tests::matmul;
use dace_tests::polybench::{
    _2mm, _3mm, cholesky, gemm, gramschmidt_trace, lu, mvt, syr2d, syrk, trisolv, trmm_trace,
};
use std::{env, time::Instant};
use tracing_subscriber::EnvFilter;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Format:   exe   test_mode   data1,data2,data3,data4,...")
    }

    let t_mode = &args[1];
    let argdata = &args[2];
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("LOG_LEVEL"))
        .init();
    // let mut wrapping_loop = Vec::new();
    let split: Vec<&str> = argdata.split(',').collect();
    let mut trace = match t_mode.as_str() {
        "lu" => lu(split[0].parse::<usize>().unwrap()),
        "trmm_trace" => trmm_trace(
            split[0].parse::<usize>().unwrap(),
            split[1].parse::<usize>().unwrap(),
        ),
        "mvt" => mvt(split[0].parse::<usize>().unwrap()),
        "trisolv" => trisolv(split[0].parse::<usize>().unwrap()),
        "syrk" => syrk(
            split[0].parse::<usize>().unwrap(),
            split[1].parse::<usize>().unwrap(),
        ),
        "syr2d" => syr2d(
            split[0].parse::<usize>().unwrap(),
            split[1].parse::<usize>().unwrap(),
        ),
        "gemm" => gemm(split[0].parse::<usize>().unwrap()),
        "cholesky" => cholesky(split[0].parse::<usize>().unwrap()),
        "gramschmidt_trace" => gramschmidt_trace(
            split[0].parse::<usize>().unwrap(),
            split[1].parse::<usize>().unwrap(),
        ),
        "3mm" => _3mm(
            split[0].parse::<usize>().unwrap(),
            split[1].parse::<usize>().unwrap(),
            split[2].parse::<usize>().unwrap(),
            split[3].parse::<usize>().unwrap(),
            split[4].parse::<usize>().unwrap(),
        ),
        "2mm" => _2mm(
            split[0].parse::<usize>().unwrap(),
            split[1].parse::<usize>().unwrap(),
            split[2].parse::<usize>().unwrap(),
            split[3].parse::<usize>().unwrap(),
        ),
        "matmul" => matmul(split[0].parse::<usize>().unwrap()),
        _ => matmul(split[0].parse::<usize>().unwrap()),
    };
    let start = Instant::now();
    let _hist = sampling::tracing_ri(&mut trace);
    let end = Instant::now();
    println!("gemm trace time: {:?}", end - start);
}
