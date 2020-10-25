mod ec2;
mod credentials;

extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;
extern crate rusoto_sts;

use rusoto_core::{HttpClient, Region};
use rusoto_ec2::{Ec2, Ec2Client};
use rusoto_ec2::{Tag, DescribeInstancesRequest, RunInstancesRequest};
use std::fs::File;
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, Sts, GetAccessKeyInfoRequest};
use rusoto_credential::ProvideAwsCredentials;
use crate::credentials::set_env::set_env_cred_from;

#[tokio::main]
async fn main() {
    let iam_cred_file = File::open("C:/Users/k3nne/Documents/aws/credentials/mc-server/new_user_credentials.csv").unwrap();
    set_env_cred_from(iam_cred_file).await;

    //REFACTOR AS FUCK HOLY SHIT THIS SOURCE CODE LOOKS SHIT AF JESUS CHRIST
    let ec2 = ec2::instance::Ec2Object::retrieve("minecraft").await.unwrap();
    ec2.status().await;
}
async fn describe(ec2: &Ec2Client){
    let describe_instances_request = DescribeInstancesRequest::default();
    match ec2.describe_instances(describe_instances_request).await {
        Ok(val) => {println!("{:?}", val)},
        Err(e) => panic!("{:?}", e)
    };
}
