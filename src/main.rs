#![feature(proc_macro_hygiene, decl_macro)]
#![feature(async_closure)]

extern crate ssh2;
extern crate ron;
extern crate rusoto_s3;
extern crate tokio;

mod mc_server;

mod aws;

use mc_server::server::*;

use std::fs::{File, read_to_string};
use std::collections::HashMap;

use crate::aws::credentials::set_env::{set_env_cred_from, set_env_cred};
use crate::aws::virtual_machine::ec2;
use crate::aws::virtual_machine::vm::{VMCore, VMNetwork};
use crate::aws::ssh::ssh_agent;

use std::net::TcpStream;
use std::path::Path;
use ssh2::{Session, Channel, ReadWindow};
use std::io::{Read, Write};
use crate::aws::ssh::ssh_agent::SSHAgent;
use ron::from_str;
use crate::aws::virtual_machine::ec2::instance::{Ec2Config, Ec2Object};
use serde::Deserialize;
use std::thread;
use rusoto_core::Region;
use rusoto_s3::{S3Client, S3, GetObjectRequest};

use std::env;

use tokio::io::AsyncReadExt;
use crate::aws::credentials::credential::Credential;

use std::str;
use rocket::http::RawStr;
use rocket::State;

use tokio::sync::Mutex;
use tokio::runtime::Runtime;


// const CONFIG_PATH: &str = "data/ec2_credentials.ron";
///dev or prod
const CREDENTIAL: &str = "dev";

fn DEP_get_config(config: &str, key: &str) -> Option<Ec2Config> {
    let mut configs: HashMap<String, Ec2Config> = from_str(config).unwrap();
    return configs.remove(key);
}

async fn DEP_get_bucket_obj(obj: &str) -> String {
    let s3 = S3Client::new(Region::UsEast2);
    let mut req = GetObjectRequest::default();
    req.bucket = env::var("AWS_BUCKET").unwrap();
    req.key = obj.parse().unwrap();
    let res = s3.get_object(req).await.unwrap();
    let mut buf = Vec::<u8>::new();
    res.body.unwrap().into_async_read().read_to_end(&mut buf).await;
    for val in &buf {
        print!("{}",*val as char);
    }
    let mut config_str = String::new();

    config_str.push_str(str::from_utf8(&*buf).unwrap());

    return config_str;
}

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "server turns off every hour. This will eventually change to 1 hour after everyone is offline\n\n\
    if the page you are trying to access is taking a while to load, the server is likely starting or stopping \n\
    and will load within 2 minutes. (I'm having issues with threads server-side and Rust is not forgiving.\n\n\
    access by going to these links example:(heroku.whatever.the.url.is.com/command/start)\n\n\
    currently implemented: \n\
    \t/command/start\n\
    \t/command/status\n\
    start will start server if off, then give ip, otherwise will immediately give ip\n\
    status will give status and ip if can get ip"
}
#[get("/command/<command>")]
async fn command(command: &RawStr, server: State<'_,Mutex<MCServer>>) -> String {
    match command.as_str() {
        "start" => {
            let status = (*server.lock().await).status().await.unwrap();

            let ip = (*server.lock().await).start().await.unwrap();

            return ip;
        },
        "status" => {
            let status = (*server.lock().await).status().await.unwrap().clone();
            if &*status == "running" {
                let ip_good = ((*server.lock().await).get_ip().await.unwrap().clone());
                let logs_good = (*server.lock().await).log().await.unwrap().clone();
                format!("status: {}\nip: {}\nlogs: {}", status, ip_good, logs_good)
            }
            else{
                format!("status: {}\nip: {}\nlogs: {}", status, "no ip not on", "could not retrieve logs")
            }
        }
        "stop" => {
            let status = (*server.lock().await).status().await.unwrap().clone();
            if &*status == "running" {
                (*server.lock().await).stop().await;
                format!("stopped!")
            }
            else {
                format!("already stopped!")
            }
        }
        _ => format!("unknown command!")
    }
}
async fn DEP_get_ec2() -> Ec2Object {
    let config_str = DEP_get_bucket_obj("ec2_credentials.ron").await;

    let config = match DEP_get_config(&*config_str, &*env::var("PRODDEV").unwrap()) {
        Some(val) => val,
        None => panic!("credential with given key not found!")
    };

    let ssh_key = DEP_get_bucket_obj("aws-ec2-test.pem").await;
    let mut file = File::create("data/aws-ec2-test.pem").unwrap();
    file.write_all(ssh_key.as_ref()).unwrap();

    println!("Credentials!");

    let mut ec2 = match ec2::instance::Ec2Object::retrieve(&*config.instance_id.as_ref().unwrap(), &*config.role_arn.as_ref().unwrap()).await {
        Some(ec2) => ec2,
        None => panic!("Couldn't find instance! Does it exist?")
    };
    return ec2;
}
async fn DEP_get_server() -> MCServer {
    let config_str = DEP_get_bucket_obj("ec2_credentials.ron").await;

    let config = match DEP_get_config(&*config_str, &*env::var("PRODDEV").unwrap()) {
        Some(val) => val,
        None => panic!("credential with given key not found!")
    };

    let ssh_key = DEP_get_bucket_obj("aws-ec2-test.pem").await;
    let mut file = File::create("data/aws-ec2-test.pem").unwrap();
    file.write_all(ssh_key.as_ref()).unwrap();

    println!("Credentials!");

    let mut ec2 = match ec2::instance::Ec2Object::retrieve(&*config.instance_id.as_ref().unwrap(), &*config.role_arn.as_ref().unwrap()).await {
        Some(ec2) => ec2,
        None => panic!("Couldn't find instance! Does it exist?")
    };

    return MCServer::new(
        ec2,
        config
    )
}

#[tokio::main]
async fn main() {
    //0:file
    //1:username
    //2:access key id
    //3:secret access key
    // let mut args: Vec<String> = env::args().collect();
    // println!("{:?}", args);

    let env_cred = Credential {
        access_key_id: env::var("AWS_ACCESS_KEY_ID").unwrap().clone(),
        secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").unwrap().clone()
    };
    set_env_cred(env_cred).await;



    let mut server_rw = Mutex::new(DEP_get_server().await);
    // let mut off_clock = Mutex::new(timer::Timer::new());
    // server.start().await;

    // println!("{}",server.log().await.unwrap());
    rocket::ignite().mount("/", routes![index, command])
        .manage(server_rw)
        // .manage(off_clock)
        .launch().await;
    // rocket::ignite()


    // run(&ssh, "cd ./minecraft && sudo nohup ./run.sh > ~/minecraft_out.txt &").await;
    // println!("status: {:?}", (ec2.status()).await);
    // loop {
    //     run(&ssh, "cat minecraft_out.txt");
    //     std::thread::sleep(std::time::Duration::from_secs(5));
    // }
}
