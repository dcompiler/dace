use stack_alg_sim::vec::LRUVec;
use stack_alg_sim::stack::LRUStack;
use std::str::FromStr;
use rusoto_s3::{PutObjectRequest, S3};
use rusoto_core::{Region, ByteStream};
use std::error::Error;
use stack_alg_sim::LRU;
use list_serializable::ListSerializable;


pub trait str_formatting { fn format_string(&self) -> String; }


impl str_formatting for usize {
    fn format_string(&self) -> String {
        self.to_string()
    }
}

impl str_formatting for (usize, usize) {
    fn format_string(&self) -> String {
        format!("{} {}", self.0, self.1)
    }
}

//Save serialized data to cloud
pub async fn save_serialized(data: &String, bucket: &str, path: &str) -> Result<(), Box<dyn Error>> {

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

// // Reads an LRU file from cloud and converts it into a histogram
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


// pub fn save_csv<T: str_formatting> (list: ListSerializable<T>, bucket: &str, key: &str) {

// }