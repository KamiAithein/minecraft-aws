extern crate ssh2;
extern crate ron;

mod mc_server;

mod aws;

use mc_server::server::*;

use std::fs::{File, read_to_string};
use std::collections::HashMap;

use crate::aws::credentials::set_env::set_env_cred_from;
use crate::aws::virtual_machine::ec2;
use crate::aws::virtual_machine::vm::{VMCore, VMNetwork};
use crate::aws::ssh::ssh_agent;

use std::net::TcpStream;
use std::path::Path;
use ssh2::{Session, Channel, ReadWindow};
use std::io::Read;
use crate::aws::ssh::ssh_agent::SSHAgent;
use ron::from_str;
use crate::aws::virtual_machine::ec2::instance::Ec2Config;
use serde::Deserialize;
use std::thread;


const CONFIG_PATH: &str = "data/ec2_credentials.ron";
///dev or prod
const CREDENTIAL: &str = "dev";

fn DEP_get_config(key: &str) -> Option<Ec2Config> {
    let mut configs: HashMap<String, Ec2Config> = from_str(&*read_to_string(CONFIG_PATH).unwrap()).unwrap();
    return configs.remove(key);
}

#[tokio::main]
async fn main() {
    println!("Start! {} ", CREDENTIAL);
    let config = match DEP_get_config(CREDENTIAL) {
        Some(val) => val,
        None => panic!("credential with given key not found!")
    };

    println!("{:?}", config);
    let iam_cred_file = File::open(config.cred_csv_path.as_ref().unwrap()).unwrap();
    set_env_cred_from(iam_cred_file).await;
    println!("Credentials!");

    let mut ec2 = match ec2::instance::Ec2Object::retrieve(&*config.instance_id.as_ref().unwrap(), &*config.role_arn.as_ref().unwrap()).await {
        Some(ec2) => ec2,
        None => panic!("Couldn't find instance! Does it exist?")
    };


    let mut server = mc_server::server::MCServer::new(ec2, config);
    // server.start().await;

    println!("{}",server.log().await.unwrap());

    //
    // // run(&ssh, "cd ./minecraft && sudo nohup ./run.sh > ~/minecraft_out.txt &").await;
    // println!("status: {:?}", (ec2.status()).await);
    // loop {
    //     run(&ssh, "cat minecraft_out.txt");
    //     std::thread::sleep(std::time::Duration::from_secs(5));
    // }
}
