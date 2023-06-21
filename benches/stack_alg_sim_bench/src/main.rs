use csv::Writer;
use rand::Rng;
use rusoto_core::{ByteStream, Region};
use rusoto_s3::{PutObjectRequest, S3};
use stack_alg_sim::stack::LRUStack;
use stack_alg_sim::vec::LRUVec;
use stack_alg_sim::LRU;
use std::env;
use std::error::Error;
use std::str::FromStr;
use std::time::Instant;
// use dace::ast::{Node, Stmt, LoopBound, AryRef};
// use dace::arybase::set_arybase;

fn generate_data(size: usize, mode: &str, flag: u8) -> Vec<String> {
    match mode {
        "Cyclic" => {
            let mut data = Vec::new();
            // for _ in 0..repeat{
            for i in 0..size {
                // println!("{:?}", i);
                data.push(i.to_string());
            }
            // }
            data
        }
        "Sawtooth" => {
            let mut data = Vec::new();
            // // for r in 0..repeat{
            if flag == 0 {
                for i in 0..size {
                    // println!("{:?}", i);
                    data.push(i.to_string());
                }
            } else {
                for i in (0..size).rev() {
                    // println!("{:?}", i);
                    data.push(i.to_string());
                }
            }
            // }
            data
        }
        "Random" => {
            let mut data = Vec::new();
            let mut rng = rand::thread_rng();
            // for _ in 0..repeat{
            for _ in 0..size {
                data.push(rng.gen_range(0..size).to_string());
            }
            // }
            data
        }
        _ => Vec::new(),
    }
}

async fn save_csv(
    data: &Vec<(String, Option<usize>)>,
    bucket: &str,
    path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_writer(vec![]);

    for i in data {
        wtr.write_record([i.0.as_str(), &i.1.unwrap_or(0).to_string()])?;
    }

    let csv_data = wtr.into_inner().unwrap();

    let region = Region::from_str("us-east-2")?;
    let s3_client = rusoto_s3::S3Client::new(region);

    let put_req = PutObjectRequest {
        bucket: bucket.to_string(),
        key: path.to_string(),
        body: Some(ByteStream::from(csv_data)),
        ..Default::default()
    };

    s3_client.put_object(put_req).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("Format:   exe   mode   test_mode   mem_size/a_size_row[MM]   data_size/a_size_col[MM]   repetitions/b_size_row[MM]   b_size_col[MM]");
        return Ok(());
    }

    let mode = &args[1];
    let t_mode = &args[2];
    let argdata = &args[3];

    let split: Vec<&str> = argdata.split(',').collect();

    let start = Instant::now();

    let dists: Vec<(String, Option<usize>)> = match t_mode.as_str() {
        // "2mm" => {
        //     dace_tests::_2mm()
        // },
        "MM" => match mode.as_str() {
            "Vec" => test_cases::nmm(
                split[0].parse::<usize>().unwrap(),
                split[1].parse::<usize>().unwrap(),
                split[2].parse::<usize>().unwrap(),
                split[3].parse::<usize>().unwrap(),
                "Vec".to_string(),
            ),
            "Stack" => test_cases::nmm(
                split[0].parse::<usize>().unwrap(),
                split[1].parse::<usize>().unwrap(),
                split[2].parse::<usize>().unwrap(),
                split[3].parse::<usize>().unwrap(),
                "Stack".to_string(),
            ),
            _ => Vec::new(),
        },
        _ => {
            // let mut total_time = Duration::new(0, 0);
            let mut flag: u8 = 0;

            // let mut total_mis = 0;

            // let data_objects: Vec<&str> = argdata.split(',').collect();
            // let mem_size = split[0].parse::<usize>().unwrap();
            let d_size = &split[1].parse::<usize>().unwrap();
            let repeat = &split[2].parse::<usize>().unwrap();

            let mut analyzer: Box<dyn LRU<String>> = if mode.as_str() == "Vec" {
                Box::new(LRUVec::<String>::new())
            } else {
                Box::new(LRUStack::<String>::new())
            };

            let mut res: Vec<(String, Option<usize>)> = Vec::new();

            for r in 0..*repeat {
                println!("repeating {} time.", r + 1);
                println!("Data generation start.");
                let data = generate_data(*d_size, t_mode, flag);
                flag ^= 1;
                // println!("Data generation finish.");
                // let all_size = &data.len();
                // let mut miss = 0;
                // match mode.as_str(){
                // "Vec" =>{
                for c in data {
                    let cur = analyzer.rec_access(c.to_string());
                    // println!("{:?}", cur);
                    res.push((c, cur));
                }

                // },
                // "Stack" =>{
                // for c in data {

                //     let cur = analyzer.rec_access(c.to_string());

                //     // println!("{:?}", cur);
                //     res.push((c, cur));
                // }
                // },
                // _ => {
                //     println!("Mode Stack or Mode Vec.");
                // },
            }

            res
        }
    };

    let duration = start.elapsed();
    // total_time += duration;
    println!("{:?}", duration);
    // println!("missing rate: {:.3?}\n", miss as f32 / *all_size as f32);
    // total_mis += miss;
    // ...

    let bucket = "lru-csv-data";
    let csv_path = mode.as_str().to_owned() + "_" + t_mode + "_" + argdata + ".csv";
    match save_csv(&dists, bucket, &csv_path).await {
        Ok(_) => {
            println!("csv path: {:?}", csv_path);
        }
        Err(m) => {
            println!("{:?}", m);
        }
    }

    // ...

    // }
    // println!("Total Time: {:?}, Avg Time: {:?}", duration, duration / *repeat as u32);
    // println!("Avg Miss Rate: {:.3?}", total_mis as f32 / (d_size * repeat) as f32);

    Ok(())
}
