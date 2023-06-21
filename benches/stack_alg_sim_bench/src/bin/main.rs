use csv::Reader;
use hist::Hist;
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3Client, S3};
use std::str::FromStr;
use tokio::io::AsyncReadExt;

use rusoto_s3::ListObjectsV2Request;

async fn process_all_csv_files(
    input_bucket: &str,
    output_bucket: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let region = Region::from_str("us-east-2")?;
    let s3_client = S3Client::new(region);

    let list_req = ListObjectsV2Request {
        bucket: input_bucket.to_string(),
        ..Default::default()
    };

    let result = s3_client.list_objects_v2(list_req).await?;
    if let Some(objects) = result.contents {
        for object in objects {
            let key = object.key.unwrap();
            if key.ends_with(".csv") {
                let hist = process_csv_data(input_bucket, &key).await?;

                let output_key = key.replace(".csv", ".txt");
                upload_histogram(output_bucket, &output_key, hist).await?;
            }
        }
    }

    Ok(())
}

async fn process_csv_data(bucket: &str, key: &str) -> Result<Hist, Box<dyn std::error::Error>> {
    let region = Region::from_str("us-east-2")?;
    let s3_client = S3Client::new(region);

    let get_req = GetObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        ..Default::default()
    };

    let result = s3_client.get_object(get_req).await?;
    let mut reader = result.body.unwrap().into_async_read();
    let mut csv_string = String::new();
    reader.read_to_string(&mut csv_string).await?;

    let mut csv_reader = Reader::from_reader(csv_string.as_bytes());

    let mut hist = Hist::new();

    for result in csv_reader.records() {
        let record = result?;
        let distance: usize = record.get(1).unwrap().parse()?;
        hist.add_dist(Some(distance));
    }

    Ok(hist)
}

async fn upload_histogram(
    bucket: &str,
    key: &str,
    hist: Hist,
) -> Result<(), Box<dyn std::error::Error>> {
    let region = Region::from_str("us-east-2")?;
    let s3_client = S3Client::new(region);

    let hist_vec = hist.to_vec();
    let hist_string = hist_vec
        .iter()
        .map(|(k, v)| format!("{:?},{}", k, v))
        .collect::<Vec<String>>()
        .join("\n");

    let put_req = PutObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        body: Some(hist_string.into_bytes().into()),
        ..Default::default()
    };

    s3_client.put_object(put_req).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    process_all_csv_files("lru-csv-data", "lru-hist-data").await
}
