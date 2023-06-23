use std::env;
use mysql::{self, OptsBuilder, Opts};

pub fn connectToDB() -> mysql::PooledConn{
    let username = env::var("db_username").expect("db_username must be set");
    let password = env::var("db_password").expect("db_password must be set");
    let endpoint = env::var("db_endpoint").expect("db_endpoint must be set");
    let name = env::var("db_name").expect("db_name must be set");

    let builder = OptsBuilder::default()
        .user(Some(username))
        .pass(Some(password))
        .ip_or_hostname(Some(endpoint))
        .db_name(Some(name));

    let opts = Opts::from(builder);

    let mysql_pool = mysql::Pool::new(opts).expect("Failed to create a MySQL Pool");

    mysql_pool.get_conn().expect("Failed to get MySQL connection")
}