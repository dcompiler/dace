CREATE DATABASE IF NOT EXISTS research;

USE research;

CREATE TABLE IF NOT EXISTS user_requests (
    email VARCHAR(100) PRIMARY KEY,
    user_name VARCHAR(100) NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    user_name VARCHAR(50) NOT NULL,
    email VARCHAR(100) PRIMARY KEY,
    access_key VARCHAR(100) UNIQUE,
    FOREIGN KEY (email) REFERENCES user_requests(email)
);

-- CREATE INDEX idx_users_email ON users (email);

CREATE TABLE IF NOT EXISTS programs (
    program_name VARCHAR(100) PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS lru_types (
    lru_type VARCHAR(25) PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS entries (
    program_name VARCHAR(100) NOT NULL,
    lru_type VARCHAR(25) NOT NULL,
    argdata VARCHAR(150) NOT NULL,
    time_elapsed FLOAT NOT NULL,
    trace_csv_link VARCHAR(150) NOT NULL,
    hist_rd_csv_link VARCHAR(150) NOT NULL,
    hist_ri_csv_link VARCHAR(150) NOT NULL,
    dist_rd_csv_link VARCHAR(150) NOT NULL,
    dist_ri_csv_link VARCHAR(150) NOT NULL,
    serialized_trace_link VARCHAR(150) NOT NULL,
    serialized_hist_rd_link VARCHAR(150) NOT NULL,
    serialized_hist_ri_link VARCHAR(150) NOT NULL,
    serialized_dist_rd_link VARCHAR(150) NOT NULL,
    serialized_dist_ri_link VARCHAR(150) NOT NULL,
    loop_code_hash VARCHAR(150) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL,
    PRIMARY KEY (program_name, lru_type, argdata),
    FOREIGN KEY (program_name) REFERENCES programs(program_name),
    FOREIGN KEY (created_by) REFERENCES users(email),
    FOREIGN KEY (lru_type) REFERENCES lru_types(lru_type)
);

