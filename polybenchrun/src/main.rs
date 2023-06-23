use aws_utilities;
use dace_tests::polybench::{_2mm, _3mm, matmul};
use list_serializable;
use mysql;
use tracer::trace::trace;
use std::{env, fmt::format};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("Format:   exe   lru_mode   test_mode   data1,data2,data3,data4,...");
        return Ok(());
    }

    let lru_mode = &args[1];
    let t_mode = &args[2];
    let argdata = &args[3];

    let conn = aws_utilities::rds::connectToDB();

    let split: Vec<&str> = argdata.split(',').collect();

    let mut loop_code = match t_mode.as_str() {
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

    let result = trace(&mut loop_code, &lru_mode);

    let serialized_access_data = serde_json::to_string(&access_data)?;
    let serialized_result_data = serde_json::to_string(&result)?;

    let bucket = "lru-data";
    let accesses_path = format!("accesses_{}_{}_{}", *t_mode, *lru_mode, *argdata);
    let hist_path = format!("hist_{}_{}_{}", *t_mode, *lru_mode, *argdata);

    aws_utilities::save_serialized(serialized_access_data, bucket, accesses_path.as_str()).await;
    aws_utilities::save_serialized(serialized_result_data, bucket, hist_path.as_str()).await;

    Ok(())
}
