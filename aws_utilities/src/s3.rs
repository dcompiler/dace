use std::env;
use std::time::{Instant};
use csv::Writer;
use rand::Rng;
use lru_vec::LRUVec;
use lru_stack::LRUStack;
use std::str::FromStr;
use rusoto_s3::{PutObjectRequest, S3};
use rusoto_core::{Region, ByteStream};
use std::error::Error;
use lru_trait::LRU;
use list_serializable::list_accesses;


//Save LRU to cloud
pub async fn save_serialized(data: String, bucket: &str, path: &str) -> Result<(), Box<dyn Error>> {

    let region = Region::from_str("us-east-2")?;
    let s3_client = rusoto_s3::S3Client::new(region);

    let put_req = PutObjectRequest {
        bucket: bucket.to_string(),
        key: path.to_string(),
        body: Some(data.into_bytes().into()),
        ..Default::default()
    };

    s3_client.put_object(put_req).await?;

    Ok(())
}

// Iterates through all LRU files in cloud, attempts to convert them into histogram, and upload histogram to cloud
pub async fn process_all_csv_files(input_bucket: &str, output_bucket: &str) -> Result<(), Box<dyn std::error::Error>> {
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

// Reads an LRU file from cloud and converts it into a histogram
pub async fn process_csv_data(bucket: &str, key: &str) -> Result<Hist, Box<dyn std::error::Error>> {
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

//Upload a histogram to cloud
pub async fn upload_histogram(bucket: &str, key: &str, hist: Hist) -> Result<(), Box<dyn std::error::Error>> {
    let region = Region::from_str("us-east-2")?;
    let s3_client = S3Client::new(region);

    let hist_vec = hist.to_vec();
    let hist_string = hist_vec.iter().map(|(k, v)| format!("{:?},{}", k, v)).collect::<Vec<String>>().join("\n");

    let put_req = PutObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        body: Some(hist_string.into_bytes().into()),
        ..Default::default()
    };

    s3_client.put_object(put_req).await?;

    Ok(())
}

