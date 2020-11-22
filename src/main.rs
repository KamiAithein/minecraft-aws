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
use std::borrow::Borrow;
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
    return NamedFile::open("static/index.xhtml").await.map_err(|e| NotFound(e.to_string()));
}
#[get("/command/<command>")]
async fn command(mut is_shutdown_queued: State<'_,Arc<AtomicBool>>, command: &RawStr) -> String {
    println!("command!");
    let mut server = DEP_get_server().await;
    println!("aaa");
    match command.as_str() {
        "start" => {
            println!("wat the fuck");
            let status = server.status().await.unwrap();

            let ip = server.start().await.unwrap();
            let ip_to_socket = ip.clone();
            // (*trigger).0.try_send(()).unwrap();
            println!("Checking if shutdown queued!");
            if !(**is_shutdown_queued).load(Ordering::Relaxed) {
                println!("Shutdown not queued, queuing!");
                is_shutdown_queued.swap(true, Ordering::Relaxed);
                let arc = (*is_shutdown_queued).clone();
                let (sender, receiver) = mpsc::sync_channel(1);
                thread::spawn(move || worker(receiver, arc, ip_to_socket, 60*60));
            }
            else {
                println!("Shutdown already queued!");
            }

            return ip;
        },
        "status" => {
            let status = server.status().await.unwrap().clone();
            if &*status == "running" {
                let ip_good = (server.get_ip().await.unwrap().clone());
                let logs_good = server.log().await.unwrap().clone();
                format!("status: {}\nip: {}\nlogs: {}", status, ip_good, logs_good)
            }
            else{
                format!("status: {}\nip: {}\nlogs: {}", status, "no ip not on", "could not retrieve logs")
            }
        }
        "stop" => {
            let status = server.status().await.unwrap().clone();
            if &*status == "running" {
                server.stop().await;
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

    ec2.start().await;

    println!("ec2 been got");

    let mut ssh: SSHAgent = loop {
        match SSHAgent::new(&ec2, Path::new(&config.ssh_key.as_ref().unwrap())).await {
            Ok(agent) => break agent,
            Err(e) => {
                // panic!("couldnt make ssh agent! Correct key?");
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        };
    };


    return MCServer::new(
        ec2,
        config,
        ssh
    )
}
// #[derive(Deserialize)]
// struct MCPlayers {
//     online: i32,
//     max: i32,
//     list: Option<Vec<String>>
// }
// #[derive(Deserialize)]
// struct MCResp {
//     ip: String,
//     port: i32,
//     online: bool,
//     #[serde(with = "serde_with::json::nested")]
//     players: Option<MCPlayers>
// }
fn worker(trigger: mpsc::Receiver<()>, mut is_shutdown_queued: Arc<AtomicBool>, socket_ip: String, sec: i32) {
    // loop {
    println!("start timer!!");
    println!("{:?}",std::time::SystemTime::now().duration_since(UNIX_EPOCH));
    println!("sleep call!");
    spin_sleep::sleep(std::time::Duration::from_secs((sec as u64)));
    println!("{:?}",std::time::SystemTime::now().duration_since(UNIX_EPOCH));
    println!("after last time!");
    //
    // let mut api = String::from("https://api.mcsrvstat.us/1/");
    // let mut ip_port = socket_ip.clone();
    // ip_port.push_str(":25565");
    // api.push_str(&*ip_port);
    // println!("{}",api);
    // let res_no_json = Runtime::new().expect("well fuck").block_on(reqwest::get(Url::from_str(&api).unwrap())).unwrap();
    // match Runtime::new().expect("well fuck").block_on(res_no_json.json::<MCResp>()) {
    //     Ok(res) => {
    //         let empty_server = res.players.is_none() || res.players.unwrap().online == 0;
    //         if empty_server {
    //             println!("NOONE!");
                let mut server = Runtime::new().expect("well fuck").block_on(DEP_get_server());
                Runtime::new().expect("well fuck").block_on(server.stop());
                is_shutdown_queued.swap(false, Ordering::Relaxed);
    //             println!("Empty server shutdown!");
    //         }
    //         else {
    //             println!("SOMEONE!!");
    //             worker(trigger, is_shutdown_queued, socket_ip, 30);
    //         }
    //     },
    //     Err(e) => {
    //         println!("ERR!");
    //         worker(trigger, is_shutdown_queued, socket_ip, 30);
    //     }
    // };
    // }
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


    // let mut off_clock = Mutex::new(timer::Timer::new());
    // server.start().await;

    // println!("{}",server.log().await.unwrap());

    let is_shutdown_queued = Arc::new(AtomicBool::new(false));

    rocket::ignite()
        .mount("/", routes![index, command])
        .manage(is_shutdown_queued)
        .launch().await;
    // rocket::ignite()


    // run(&ssh, "cd ./minecraft && sudo nohup ./run.sh > ~/minecraft_out.txt &").await;
    // println!("status: {:?}", (ec2.status()).await);
    // loop {
    //     run(&ssh, "cat minecraft_out.txt");
    //     std::thread::sleep(std::time::Duration::from_secs(5));
    // }
}
