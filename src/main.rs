#![feature(proc_macro_hygiene, decl_macro)]
#![feature(async_closure)]
extern crate ssh2;
extern crate ron;
extern crate rusoto_s3;
extern crate tokio;
extern crate reqwest;

mod mc_server;

mod aws;

use mc_server::server::*;

use std::fs::{File, read_to_string};
use std::collections::HashMap;

use crate::aws::credentials::set_env::{set_env_cred_from, set_env_cred};
use crate::aws::virtual_machine::ec2;
use crate::aws::virtual_machine::vm::{VMCore, VMNetwork};
use crate::aws::ssh::ssh_agent;

use mc_server::server::State as MCState;

use std::net::TcpStream;
use std::path::Path;
use ssh2::{Session, Channel, ReadWindow};
use std::io::{Read, Write};
use crate::aws::ssh::ssh_agent::SSHAgent;
use ron::from_str;
use crate::aws::virtual_machine::ec2::instance::{Ec2Config, Ec2Object};
use serde::Deserialize;
use std::{thread, io};
use rusoto_core::Region;
use rusoto_s3::{S3Client, S3, GetObjectRequest};

use std::env;

use tokio::io::AsyncReadExt;
use crate::aws::credentials::credential::Credential;

use std::str;
use rocket::http::RawStr;
use rocket::{State, futures};

use tokio::sync::Mutex;
use tokio::runtime::Runtime;
use rocket::response::NamedFile;
use rocket::response::status::NotFound;
use rocket::fairing::AdHoc;
use std::sync::{mpsc, Arc};
use std::time::UNIX_EPOCH;

use std::sync::atomic::{AtomicBool, Ordering};
use std::borrow::{Borrow, BorrowMut};
use std::error::Error;
use futures::executor::block_on;
use reqwest::Url;
use std::str::FromStr;


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

    let mut config_str = String::new();

    config_str.push_str(str::from_utf8(&*buf).unwrap());

    return config_str;
}

#[macro_use] extern crate rocket;

#[get("/")]
async fn index() -> Result<NamedFile, NotFound<String>> {
    return NamedFile::open("static/index.html").await.map_err(|e| NotFound(e.to_string()));
}
#[get("/command/<command>")]
async fn command(mut is_shutdown_queued: State<'_,Arc<AtomicBool>>, mut server_arc: State<'_,Arc<Mutex<MCServer>>>, command: &RawStr) -> String {
    println!("command!");
    let mut server= (*server_arc).clone();
    println!("aaa");
    let status: MCState = server.lock().await.status().await.clone();
    match command.as_str() {
        "start" => {
            let ip = server.lock().await.start().await.unwrap().clone();

            return ip.to_string();
        },
        "status" => {
            match status {
                MCState::STARTED  => {
                    let ip_good = (server.lock().await.get_ip().await.unwrap().clone());
                    let logs_good = server.lock().await.log().await.unwrap().clone();
                    format!("status: {}<br/>ip: {}<br/>logs: {}", status, ip_good, logs_good)
                }
                MCState::STARTING => {
                    "starting!".to_string()
                }
                _ =>format!("status: {}<br/>ip: {}<br/>logs: {}", status, "no ip not on", "could not retrieve logs")


            }
        }
        "stop" => {
            match status {
                MCState::STARTED  => {
                    server.lock().await.stop().await;
                    format!("stopped!")
                }
                MCState::STARTING => {
                    server.lock().await.stop().await;
                    format!("stopped!")
                }
                _ =>format!("already stopped!")


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
    let config_str = match &*env::var("LOCAL").unwrap() {
        "true" => std::fs::read_to_string("data_priv/ec2_credentials.ron").unwrap(),
        _ => DEP_get_bucket_obj("ec2_credentials.ron").await
    };

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
        config,
        None
    ).await
}

#[derive(Deserialize, Debug, Clone)]
struct MCPlayers {
    online: i32,
    max: i32
}
#[derive(Deserialize, Debug, Clone)]
struct MCResp {
    ip: String,
    port: i32,
    online: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    players: Option<MCPlayers>
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

    let is_shutdown_queued = Arc::new(AtomicBool::new(false));

    println!("making server!");
    println!("{:?}",std::time::SystemTime::now().duration_since(UNIX_EPOCH));
    let mut server = DEP_get_server().await;
    println!("{:?}",std::time::SystemTime::now().duration_since(UNIX_EPOCH));
    println!("made server!");
    let server_arc = Arc::new(Mutex::new(server));

    rocket::ignite()
        .mount("/", routes![index, command])
        .manage(is_shutdown_queued)
        .manage((server_arc))
        .launch().await;
    // rocket::ignite()



}
