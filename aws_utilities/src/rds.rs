use mysql::{self, params, prelude::Queryable, Opts, OptsBuilder};
use std::env;
use urlencoding::encode;

pub fn connect_to_db() -> mysql::PooledConn {
    let username = env::var("db_username").expect("db_username must be set");
    let password = env::var("db_password").expect("db_password must be set");
    let endpoint = env::var("db_endpoint").expect("db_endpoint must be set");
    let name = env::var("db_name").expect("db_name must be set");
    println!("{}", username);
    println!("{}", password);
    println!("{}", endpoint);
    println!("{}", name);

    // encode()

    let port = 3306; // MySQL default port

    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        username, password, endpoint, port, name
    );

    let opts = Opts::from_url(&url)
        .expect("Failed to parse MySQL URL");

    
    let mysql_pool = mysql::Pool::new(opts).expect("Failed to create a MySQL Pool");

    mysql_pool
        .get_conn()
        .expect("Failed to get MySQL connection")
}

pub async fn entry_exists(
    conn: &mut mysql::PooledConn,
    program: &str,
    lru_type: &str,
    argdata: &str,
) -> Result<bool, mysql::Error> {
    let result: Option<(u32,)> = conn.exec_first(
        "SELECT 1 FROM entries WHERE program_name = :program AND lru_type = :lru_type AND argdata = :argdata",
        params! {
            "program" => program.to_string(),
            "lru_type" => lru_type.to_string(),
            "argdata" => argdata.to_string()
        }
    )?;

    Ok(result.is_some())
}

pub fn save_entry(
    conn: &mut mysql::PooledConn,
    params: (
        &String,
        &String,
        &String,
        &String,
        &str,
        &str,
        &str,
        &str,
        &str,
        &str,
        &str,
        &str,
        &str,
        &str,
        &String,
        &String,
    ),
) -> Result<(), mysql::Error> {
    conn.exec_drop(
        r"INSERT INTO entries 
        (
            program_name, 
            lru_type, 
            argdata,
            time_elapsed, 
            trace_csv_link,
            hist_rd_csv_link,
            hist_ri_csv_link,
            dist_rd_csv_link,
            dist_ri_csv_link,
            serialized_trace_link,
            serialized_hist_rd_link,
            serialized_hist_ri_link,
            serialized_dist_rd_link,
            serialized_dist_ri_link,
            loop_code_hash,
            created_by
        ) 
        VALUES 
        (
            :program, 
            :lru_type, 
            :argdata, 
            :time_elapsed,
            :trace_csv_link,
            :hist_rd_csv_link,
            :hist_ri_csv_link,
            :dist_rd_csv_link,
            :dist_ri_csv_link,
            :serialized_trace_link,
            :serialized_hist_rd_link,
            :serialized_hist_ri_link,
            :serialized_dist_rd_link,
            :serialized_dist_ri_link,
            :loop_code_hash,
            :created_by
        )",
        params! {
            "program" => params.0,
            "lru_type" => params.1,
            "argdata" => params.2,
            "time_elapsed" => params.3,
            "trace_csv_link" => params.4,
            "hist_rd_csv_link" => params.5,
            "hist_ri_csv_link" => params.6,
            "dist_rd_csv_link" => params.7,
            "dist_ri_csv_link" => params.8,
            "serialized_trace_link" => params.9,
            "serialized_hist_rd_link" => params.10,
            "serialized_hist_ri_link" => params.11,
            "serialized_dist_rd_link" => params.12,
            "serialized_dist_ri_link" => params.13,
            "loop_code_hash" => params.14,
            "created_by" => params.15
        },
    )?;
    Ok(())
}
