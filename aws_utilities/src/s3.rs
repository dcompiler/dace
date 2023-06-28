use std::str::FromStr;
use rusoto_s3::{PutObjectRequest, S3, S3Client};
use rusoto_core::{Region, ByteStream};
// use std::error::Error;
use list_serializable::ListSerializable;
use csv::Writer;
use hist::Hist;
use anyhow::Result;

//Save serialized data to cloud
pub async fn save_serialized(data: &String, bucket: &str, path: &str) -> Result<()> {

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

// Reads an LRU file from cloud
// pub async fn process_csv_data(bucket: &str, key: &str) -> Result<Hist, Box<dyn std::error::Error>> {
//     let region = Region::from_str("us-east-2")?;
//     let s3_client = S3Client::new(region);

//     let get_req = GetObjectRequest {
//         bucket: bucket.to_string(),
//         key: key.to_string(),
//         ..Default::default()
//     };

//     let result = s3_client.get_object(get_req).await?;
//     let mut reader = result.body.unwrap().into_async_read();
//     let mut csv_string = String::new();
//     reader.read_to_string(&mut csv_string).await?;

//     let mut csv_reader = Reader::from_reader(csv_string.as_bytes());

//     let mut hist = Hist::new();

//     for result in csv_reader.records() {
//         let record = result?;
//         let distance: usize = record.get(1).unwrap().parse()?;
//         hist.add_dist(Some(distance));
//     }

//     Ok(hist)
// }


//Redundant code here
pub async fn save_csv_list_trace(list: ListSerializable<usize>, bucket: &str, key: String) -> Result<()> {
    let key = key.as_str();
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

pub async fn save_csv_list_dist(list: ListSerializable<(usize, Option<usize>)>, bucket: &str, key: String) -> Result<()>{
    let key = key.as_str();
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

pub async fn save_csv_hist(histogram: Hist, bucket: &str, key: String) -> Result<()> {
    let key = key.as_str();
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
