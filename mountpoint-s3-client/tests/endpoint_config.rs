#![cfg(feature = "s3_tests")]

pub mod common;

use aws_sdk_s3::primitives::ByteStream;
use bytes::Bytes;
use common::*;
use mountpoint_s3_client::ObjectClient;
use mountpoint_s3_client::config::{AddressingStyle, EndpointConfig, S3ClientConfig};
use mountpoint_s3_client::types::GetObjectParams;
use test_case::test_case;

async fn run_test(endpoint_config: EndpointConfig, prefix: &str, bucket: String) {
    let sdk_client = get_test_sdk_client().await;

    // Create one object named "hello"
    let key = format!("{prefix}hello");
    let body = b"hello world!";
    sdk_client
        .put_object()
        .bucket(&bucket)
        .key(&key)
        .body(ByteStream::from(Bytes::from_static(body)))
        .send()
        .await
        .unwrap();

    let config = S3ClientConfig::new().endpoint_config(endpoint_config.clone());
    let client = get_test_client_with_config(config);

    let result = client
        .get_object(&bucket, &key, &GetObjectParams::new())
        .await
        .expect("get_object should succeed");
    check_get_result(result, None, &body[..]).await;
}

#[test_case(AddressingStyle::Automatic, "test_default_addressing_style")]
#[test_case(AddressingStyle::Path, "test_path_addressing_style")]
#[tokio::test]
async fn test_addressing_style(addressing_style: AddressingStyle, prefix: &str) {
    run_test(
        get_test_endpoint_config().addressing_style(addressing_style),
        &get_unique_test_prefix(prefix),
        get_test_bucket(),
    )
    .await;
}

// We're not testing S3 Express One Zone against FIPS endpoints
#[cfg(not(feature = "s3express_tests"))]
#[cfg(feature = "fips_tests")]
#[tokio::test]
async fn test_use_fips() {
    let prefix = get_unique_test_prefix("test_fips");
    run_test(get_test_endpoint_config().use_fips(true), &prefix, get_test_bucket()).await;
}

// S3 Express One Zone does not support S3 Transfer Acceleration
#[cfg(not(feature = "s3express_tests"))]
// Transfer acceleration do not work with path style
#[tokio::test]
async fn test_use_accelerate() {
    let prefix = get_unique_test_prefix("test_transfer_acceleration");
    run_test(
        get_test_endpoint_config().use_accelerate(true),
        &prefix,
        get_test_bucket(),
    )
    .await;
}

// S3 Express One Zone does not support Dual-stack endpoints
#[cfg(not(feature = "s3express_tests"))]
#[test_case(AddressingStyle::Automatic, "test_dual_stack")]
#[test_case(AddressingStyle::Path, "test_dual_stack_path_style")]
#[tokio::test]
async fn test_addressing_style_dualstack_option(addressing_style: AddressingStyle, prefix: &str) {
    let prefix = get_unique_test_prefix(prefix);
    run_test(
        get_test_endpoint_config()
            .addressing_style(addressing_style)
            .use_dual_stack(true),
        &prefix,
        get_test_bucket(),
    )
    .await;
}

// We're not testing S3 Express One Zone against FIPS endpoints
#[cfg(not(feature = "s3express_tests"))]
#[cfg(feature = "fips_tests")]
#[tokio::test]
async fn test_fips_dual_stack_mount_option() {
    let prefix = get_unique_test_prefix("test_fips_dual_stack");
    run_test(
        get_test_endpoint_config().use_fips(true).use_dual_stack(true),
        &prefix,
        get_test_bucket(),
    )
    .await;
}

// S3 Express One Zone does not support access points
#[cfg(not(feature = "s3express_tests"))]
#[test_case(AddressingStyle::Automatic, true, "test_accesspoint_arn")]
#[test_case(AddressingStyle::Automatic, false, "test_accesspoint_alias")]
#[test_case(AddressingStyle::Path, false, "test_accesspoint_alias")]
// Path-style addressing cannot be used with ARN buckets for the endpoint resolution
// Also, path-style addressing is not supported for Access Points. But it seems to be supported for single region access point for now.
#[tokio::test]
async fn test_single_region_access_point(addressing_style: AddressingStyle, arn: bool, prefix: &str) {
    run_test(
        get_test_endpoint_config().addressing_style(addressing_style),
        &get_unique_test_prefix(prefix),
        get_test_access_point(arn, AccessPointType::SingleRegion),
    )
    .await;
}

#[cfg(not(feature = "s3express_tests"))]
// For Object Lambda Access Point, PutObject is not supported,
// For multi region access points, Rust SDK is not supported. Hence different helper method for these tests.
async fn run_list_objects_test(endpoint_config: EndpointConfig, prefix: &str, bucket: &str) {
    let config = S3ClientConfig::new().endpoint_config(endpoint_config.clone());
    let client = get_test_client_with_config(config);

    client
        .list_objects(bucket, None, "/", 10, prefix)
        .await
        .expect("list_object should succeed");
}

// S3 Express One Zone does not support access points
#[cfg(not(feature = "s3express_tests"))]
#[test_case(false, "test_OLAP_alias")]
#[test_case(true, "test_OLAP_ARN")]
// Path-style addressing is not supported for Access points
#[tokio::test]
async fn test_object_lambda_access_point(arn: bool, prefix: &str) {
    run_list_objects_test(
        get_test_endpoint_config(),
        &get_unique_test_prefix(prefix),
        &get_test_access_point(arn, AccessPointType::ObjectLambda),
    )
    .await;
}

// S3 Express One Zone does not support multi-region access points
#[cfg(not(feature = "s3express_tests"))]
// Path-style addressing is not supported for Access points
// Only ARN is supported for Multi Region access point as AWS CLI.
#[tokio::test]
async fn test_multi_region_access_point() {
    let prefix = "test_MRAP";
    run_list_objects_test(
        get_test_endpoint_config(),
        &get_unique_test_prefix(prefix),
        &get_test_access_point(true, AccessPointType::MultiRegion),
    )
    .await;
}
