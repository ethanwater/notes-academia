//ref: https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/struct.Client.html

use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_s3 as s3;
use s3::operation::put_object::PutObjectError;
use s3::primitives::ByteStream;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

async fn fetch_bucket_objects(
    client: &s3::Client,
    bucket: &str,
) -> Result<Vec<aws_sdk_s3::types::Object>, s3::Error> {
    let mut response = client
        .list_objects_v2()
        .bucket(bucket.to_owned())
        .into_paginator()
        .send();

    while let Some(result) = response.next().await {
        match result {
            Ok(output) => {
                return Ok(output.contents().to_vec());
            }
            Err(err) => {
                eprintln!("{err:?}");
            }
        }
    }

    let vacant_bucket: Vec<aws_sdk_s3::types::Object> = Vec::new();
    Ok(vacant_bucket)
}

fn present_bucket_objects(objects: &Vec<aws_sdk_s3::types::Object>) {
    for object in objects {
        println!(
            "{:<40} | {:>20} | {:>10} bytes",
            object.key().unwrap_or("unknown"),
            object
                .last_modified()
                .unwrap_or(&aws_sdk_s3::primitives::DateTime::from_secs(0))
                .fmt(aws_sdk_s3::primitives::DateTimeFormat::DateTime)
                .unwrap_or(String::from("unknown"))
                .as_str(),
            object.size().unwrap_or(0),
        );
    }
}

async fn download_bucket_object(
    client: &s3::Client,
    bucket: &str,
    object: &aws_sdk_s3::types::Object,
) -> Result<(), aws_sdk_s3::primitives::ByteStreamError> {
    let object_key = object.key().unwrap();
    let response = client
        .get_object()
        .bucket(bucket)
        .key(object_key)
        .send()
        .await;

    match response {
        Ok(result) => {
            dbg!(&result);
            let stream = result.body.collect().await?.into_bytes();
            {
                let mut file = File::create(object_key)?;
                file.write_all(&stream)?;
            }
        }
        Err(err) => {
            eprintln!("{:?}", err);
        }
    }

    Ok(())
}

async fn upload_bucket_object(
    client: &s3::Client,
    bucket: &str,
    file: &str,
) -> Result<(), aws_sdk_s3::primitives::ByteStreamError> {
    let stream = ByteStream::from_path(Path::new(file)).await;
    let response = client
        .put_object()
        .bucket(bucket)
        .key(file)
        .body(stream.unwrap())
        .send()
        .await;

    match response {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{:?}", err);
        }
    }

    Ok(())
}

async fn delete_bucket_object(
    client: &s3::Client,
    bucket: &str,
    object: &aws_sdk_s3::types::Object,
) -> Result<(), s3::Error> {
    let response = client
        .delete_object()
        .bucket(bucket)
        .key(object.key().unwrap())
        .send()
        .await;

    match response {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{:?}", err);
        }
    }

    Ok(())
}

#[::tokio::main]
async fn main() -> Result<(), s3::Error> {
    let aws_bucket = env::var("AWS_BUCKET").expect("AWS_BUCKET not found");
    let _aws_region = env::var("AWS_REGION").expect("AWS_REGION not found");
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let client = s3::Client::new(&config);
    let _objects = fetch_bucket_objects(&client, &aws_bucket).await?;

    Ok(())
}
