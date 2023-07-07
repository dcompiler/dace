use dace_tests::matmul;
use dace_tests::polybench::{
    _2mm, _3mm, cholesky, gemm, gramschmidt_trace, lu, mvt, syr2d, syrk, trisolv, trmm_trace,
};
use static_rd::trace::trace;
use std::sync::Arc;
use std::{env, time::Duration, time::Instant};

fn duration_to_string(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{} hours, {} minutes, {} seconds", hours, minutes, seconds)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 6 {
        println!("Format:   exe   lru_mode   test_mode   data1,data2,data3,data4,...");
        return Ok(());
    }

    let lru_mode = &args[1];
    let t_mode = &args[2];
    let creator = &args[3];
    let hash_code = &args[4]; //1.0 for now
    let argdata = &args[5];
    let skip = &args[6]; //typically should be no
                         // let data_collection = &args[7]; //choose both
    println!("{}", lru_mode);
    println!("{}", t_mode);
    println!("{}", creator);
    println!("{}", hash_code);
    println!("{}", argdata);
    println!("{}", skip);
    let mut conn = aws_utilities::rds::connect_to_db();

    if skip != "yes"
        && aws_utilities::rds::entry_exists(&mut conn, t_mode, lru_mode, argdata).await?
    {
        println!("Entry already exists. Aborting.");
        return Ok(());
    }

    let split: Vec<&str> = argdata.split(',').collect();

    let mut loop_code = match t_mode.as_str() {
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

    let result = trace(&mut loop_code, lru_mode);

    let time_elapsed = start.elapsed();

    let serialized_hist_rd_data = Arc::new(serde_json::to_string(&result.0)?);
    // let serialized_hist_ri_data = Arc::new(serde_json::to_string(&result.1)?);
    let serialized_dist_rd_data = Arc::new(serde_json::to_string(&result.1)?);
    // let serialized_dist_ri_data = Arc::new(serde_json::to_string(&result.3)?);
    let serialized_trace_data = Arc::new(serde_json::to_string(&result.2)?);

    let serialized_bucket = "serialized-data-dace";
    let csv_bucket = "csv-data-dace";

    let trace_path_json = Arc::new(format!("trace/{}_{}_{}.json", *t_mode, *lru_mode, *argdata));
    let hist_rd_path_json = Arc::new(format!(
        "hist/rd/{}_{}_{}.json",
        *t_mode, *lru_mode, *argdata
    ));
    // let hist_ri_path_json = Arc::new(format!(
    //     "hist/ri/{}_{}_{}.json",
    //     *t_mode, *lru_mode, *argdata
    // ));
    let dist_rd_path_json = Arc::new(format!(
        "dist/rd/{}_{}_{}.json",
        *t_mode, *lru_mode, *argdata
    ));
    // let dist_ri_path_json = Arc::new(format!(
    //     "dist/ri/{}_{}_{}.json",
    //     *t_mode, *lru_mode, *argdata
    // ));

    let trace_path_csv = Arc::new(format!("trace/{}_{}_{}.csv", *t_mode, *lru_mode, *argdata));
    let hist_rd_path_csv = Arc::new(format!(
        "hist/rd/{}_{}_{}.csv",
        *t_mode, *lru_mode, *argdata
    ));
    // let hist_ri_path_csv = Arc::new(format!(
    //     "hist/ri/{}_{}_{}.csv",
    //     *t_mode, *lru_mode, *argdata
    // ));
    let dist_rd_path_csv = Arc::new(format!(
        "dist/rd/{}_{}_{}.csv",
        *t_mode, *lru_mode, *argdata
    ));
    // let dist_ri_path_csv = Arc::new(format!(
    //     "dist/ri/{}_{}_{}.csv",
    //     *t_mode, *lru_mode, *argdata
    // ));
    //CSV: hist ri, hist rd, dist ri, dist rd, trace
    //serialized: hist ri, hist rd, dist ri, dist rd, trace

    // Spawn all async tasks first
    let handle1 = tokio::spawn({
        let serialized_trace_data = Arc::clone(&serialized_trace_data);
        let trace_path_json = Arc::clone(&trace_path_json);

        async move {
            let serialized_trace_data = &serialized_trace_data;
            let trace_path_json = &trace_path_json;

            aws_utilities::s3::save_serialized(
                serialized_trace_data,
                serialized_bucket,
                trace_path_json,
            )
            .await
        }
    });

    let handle2 = tokio::spawn({
        let serialized_hist_rd_data = Arc::clone(&serialized_hist_rd_data);
        let hist_rd_path_json = Arc::clone(&hist_rd_path_json);
        async move {
            let serialized_hist_rd_data = &serialized_hist_rd_data;
            let hist_rd_path_json = &hist_rd_path_json;
            aws_utilities::s3::save_serialized(
                serialized_hist_rd_data,
                serialized_bucket,
                hist_rd_path_json,
            )
            .await
        }
    });

    // let handle3 = tokio::spawn({
    //     let serialized_hist_ri_data = Arc::clone(&serialized_hist_ri_data);
    //     let hist_ri_path_json = Arc::clone(&hist_ri_path_json);
    //     async move {
    //         let serialized_hist_ri_data = &serialized_hist_ri_data;
    //         let hist_ri_path_json = &hist_ri_path_json;
    //         aws_utilities::s3::save_serialized(
    //             serialized_hist_ri_data,
    //             serialized_bucket,
    //             hist_ri_path_json,
    //         )
    //         .await
    //     }
    // });

    let handle4 = tokio::spawn({
        let serialized_dist_rd_data = Arc::clone(&serialized_dist_rd_data);
        let dist_rd_path_json = Arc::clone(&dist_rd_path_json);
        async move {
            let serialized_dist_rd_data = &serialized_dist_rd_data;
            let dist_rd_path_json = &dist_rd_path_json;
            aws_utilities::s3::save_serialized(
                serialized_dist_rd_data,
                serialized_bucket,
                dist_rd_path_json,
            )
            .await
        }
    });

    // let handle5 = tokio::spawn({
    //     let serialized_dist_ri_data = Arc::clone(&serialized_dist_ri_data);
    //     let dist_ri_path_json = Arc::clone(&dist_ri_path_json);
    //     async move {
    //         let serialized_dist_ri_data = &serialized_dist_ri_data;
    //         let dist_ri_path_json = &dist_ri_path_json;
    //         aws_utilities::s3::save_serialized(
    //             serialized_dist_ri_data,
    //             serialized_bucket,
    //             dist_ri_path_json,
    //         )
    //         .await
    //     }
    // });

    let handle6 = tokio::spawn({
        let hist_rd_path_csv = Arc::clone(&hist_rd_path_csv);
        async move { aws_utilities::s3::save_csv_hist(result.0, csv_bucket, &hist_rd_path_csv).await }
    });

    // let handle7 = tokio::spawn({
    //     let hist_ri_path_csv = Arc::clone(&hist_ri_path_csv);
    //     async move { aws_utilities::s3::save_csv_hist(result.1, csv_bucket, &hist_ri_path_csv).await }
    // });

    let handle8 = tokio::spawn({
        let dist_rd_path_csv = Arc::clone(&dist_rd_path_csv);
        async move {
            aws_utilities::s3::save_csv_list_dist(result.1, csv_bucket, &dist_rd_path_csv).await
        }
    });

    // let handle9 = tokio::spawn({
    //     let dist_ri_path_csv = Arc::clone(&dist_ri_path_csv);
    //     async move {
    //         aws_utilities::s3::save_csv_list_dist(result.3, csv_bucket, &dist_ri_path_csv).await
    //     }
    // });

    let handle10 = tokio::spawn({
        let trace_path_csv = Arc::clone(&trace_path_csv);
        async move {
            aws_utilities::s3::save_csv_list_trace(result.2, csv_bucket, &trace_path_csv).await
        }
    });

    // Store all handles in a Vec
    let handles = vec![handle1, handle2, handle4, handle6, handle8, handle10];

    // Await them all
    for handle in handles {
        handle.await??; // Use '?' if the functions return Result<_, _>
    }

    aws_utilities::rds::save_entry(
        &mut conn,
        (
            t_mode,
            lru_mode,
            argdata,
            &duration_to_string(time_elapsed),
            &trace_path_csv,
            &hist_rd_path_csv,
            // &hist_ri_path_csv,
            "NULL",
            &dist_rd_path_csv,
            // &dist_ri_path_csv,
            "NULL",
            &trace_path_json,
            &hist_rd_path_json,
            // &hist_ri_path_json,
            "NULL",
            &dist_rd_path_json,
            // &dist_ri_path_json,
            "NULL",
            hash_code,
            creator,
        ),
    )?;

    Ok(())
}
