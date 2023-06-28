use std::str::FromStr;
use rusoto_s3::{PutObjectRequest, S3, S3Client};
use rusoto_core::{Region, ByteStream};
use list_serializable::ListSerializable;
use csv::Writer;
use hist::Hist;
use anyhow::Result;

//Save serialized data to cloud
pub async fn save_serialized(data: &String, bucket: &str, path: &String) -> Result<()> {

    let region = Region::from_str("us-east-2")?;
    let s3_client = rusoto_s3::S3Client::new(region);

    let stream = ByteStream::from(data.as_bytes().to_vec());

    let put_req = PutObjectRequest {
        bucket: bucket.to_string(),
        key: path.to_string(),
        body: Some(stream),
        ..Default::default()
    };

    s3_client.put_object(put_req).await?;

    Ok(())
}

//Redundant code here
pub async fn save_csv_list_trace(list: ListSerializable<usize>, bucket: &str, key: &String) -> Result<()> {
    let region = Region::from_str("us-east-2")?;
    let s3_client = rusoto_s3::S3Client::new(region);

    let mut wtr = Writer::from_writer(vec![]);

    for i in list.get_vec() {
        wtr.write_field(&i.to_string())?;
    }

    let csv_data = wtr.into_inner().unwrap();

    let put_req = PutObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        body: Some(ByteStream::from(csv_data)),
        ..Default::default()
    };

    s3_client.put_object(put_req).await?;

    Ok(())
}

pub async fn save_csv_list_dist(list: ListSerializable<(usize, Option<usize>)>, bucket: &str, key: &String) -> Result<()>{
    let region = Region::from_str("us-east-2")?;
    let s3_client = rusoto_s3::S3Client::new(region);

    let mut wtr = Writer::from_writer(vec![]);

    for i in list.get_vec() {
        wtr.write_record(&[i.0.to_string(), i.0.to_string()])?;
    }

    let csv_data = wtr.into_inner().unwrap();

    let put_req = PutObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        body: Some(ByteStream::from(csv_data)),
        ..Default::default()
    };

    s3_client.put_object(put_req).await?;

    Ok(())
}

pub async fn save_csv_hist(histogram: Hist, bucket: &str, key: &String) -> Result<()> {
    let region = Region::from_str("us-east-2")?; 
    let s3_client = S3Client::new(region);

    let hist_vec = histogram.to_vec(); 
    let hist_string = hist_vec.iter().map(|(k, v)| format!("{},{}", k.map_or_else(|| "".to_string(), |v| v.to_string()), v.to_string())).collect::<Vec<String>>().join("\n");

    let put_req = PutObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        body: Some(hist_string.into_bytes().into()),
        ..Default::default()
    };

    s3_client.put_object(put_req).await?;

    Ok(())
}
