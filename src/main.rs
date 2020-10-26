extern crate ssh2;

use std::fs::File;

use rust_ec2::credentials::set_env::set_env_cred_from;
use rust_ec2::virtual_machine::ec2;
use rust_ec2::virtual_machine::vm::{VMCore, VMNetwork};
use rust_ec2::ssh::ssh_agent;

use std::net::TcpStream;
use std::path::Path;
use ssh2::Session;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let iam_cred_file = File::open("C:/Users/k3nne/Documents/aws/credentials/mc-server/new_user_credentials.csv").unwrap();
    set_env_cred_from(iam_cred_file).await;

    let mut ec2 = ec2::instance::Ec2Object::retrieve().await.unwrap();
    ec2.status().await;
    ec2.start().await;

   let ssh = match ssh_agent::SSHAgent::new(&ec2, Path::new("C:/Users/k3nne/Documents/aws/credentials/default/aws-ec2-test.pem")).await {
       Ok(val) => val,
       Err(e) => panic!("couldn't make ssh agent!")
   };

    println!("{}",ssh.execute("touch this_worked_again").await);

    ec2.stop().await;

    Ok(())
}
