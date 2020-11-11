extern crate ssh2;
extern crate ron;
extern crate async_trait;


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
use crate::aws::virtual_machine::ec2::instance::{Ec2Config, Ec2Object};
use serde::Deserialize;
use std::thread;
use std::error::Error;

use async_trait::async_trait;



fn DEP_on_start(ssh: &SSHAgent, startup: &Vec<String>) {
    println!("start!");
    if !DEP_exec(&ssh, "ls ~/minecraft").contains("server.jar") {
        println!("making minecraft!");
        for command in startup {
            DEP_run(ssh, &**command);
        }
    }
}
fn DEP_on_main(ssh: &SSHAgent, main: &Vec<String>) {

}
fn DEP_exec(ssh: &SSHAgent, cmd: &str) -> String {
    return ssh.execute(cmd);
}
fn DEP_run(ssh: &SSHAgent, cmd: &str) {
    println!("cmd: {}, result: {}", cmd, ssh.execute(cmd));
}


///a mc server
#[async_trait]
pub trait Server {
    ///Start the given mc server
    async fn start(&mut self) -> Result<(), Box<dyn Error>>;
    ///Stop the given mc server
    async fn stop(&mut self) -> Result<(), Box<dyn Error>>;
    ///Execute the mc command given on the server, returning the server response if no error
    async fn command(&mut self, cmd: &str) -> Result<String, Box<dyn Error>>;
    ///Get the current log of the server
    async fn log(&self) -> Result<String, Box<dyn Error>>;
}
pub struct MCServer {
    ec2: Ec2Object,
    config: Ec2Config,
}

impl MCServer {
    pub fn new(ec2: Ec2Object, config: Ec2Config) -> MCServer {
        MCServer {
            ec2,
            config,
        }
    }
}

#[async_trait]
impl Server for MCServer {
    async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        println!("ec2!");
        let status = match self.ec2.status().await {
            Some(val) => val,
            None => panic!("No status! Correct id?")
        };
        match &status[..] {
            "stopped" => {
                self.ec2.start().await;
            }
            "stopping" => {
                while self.ec2.status().await.is_none() || self.ec2.status().await.unwrap() == "stopping" {}
                self.ec2.start().await;
            }
            _ => {}
        }
        let ssh: SSHAgent = loop {
            match SSHAgent::new(&self.ec2, Path::new(&self.config.ssh_key.as_ref().unwrap())).await {
                Ok(agent) => break agent,
                Err(e) => {
                    // panic!("couldnt make ssh agent! Correct key?");
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            };
        };

        println!("ssh threads!!");
        
        let init: Vec<String> = self.config.init_script.as_ref().unwrap().clone();
        let main: Vec<String> = self.config.main_script.as_ref().unwrap().clone();

        // thread::spawn(move ||{
            println!("main!");
            if !DEP_exec(&ssh, "ps -aux").contains("minecraft") {
                println!("running main script!");
                for command in main {
                    ssh.execute(&*command);
                    // run(ssh, &**command).await;
                }
            }
            println!("start!");
            if !DEP_exec(&ssh, "ls ~/minecraft").contains("server.jar") {
                println!("making minecraft!");
                for command in init {
                    DEP_run(&ssh, &*command);
                }
            }
        // });
        // DEP_on_start(&ssh, &init.clone());
        // DEP_on_main(&ssh, &main.clone());

        return Ok(());//TODO actual return
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        let ssh: SSHAgent = loop {
            match SSHAgent::new(&self.ec2, Path::new("C:/Users/k3nne/Documents/aws/credentials/default/aws-ec2-test.pem")).await {
                Ok(agent) => break agent,
                Err(e) => {
                    // panic!("couldnt make ssh agent! Correct key?");
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            };
        };
        self.command("stop").await;
        DEP_run(&ssh, &*format!("sudo screen -S mc -X quit"));
        Ok(()) //TODO actual return
    }

    async fn command(&mut self, cmd: &str) -> Result<String, Box<dyn Error>> {
        let ssh: SSHAgent = loop {
            match SSHAgent::new(&self.ec2, Path::new("C:/Users/k3nne/Documents/aws/credentials/default/aws-ec2-test.pem")).await {
                Ok(agent) => break agent,
                Err(e) => {
                    // panic!("couldnt make ssh agent! Correct key?");
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            };
        };
        DEP_run(&ssh, &*format!("sudo screen -S mc -X stuff \"{}^M\"", cmd));
        Ok("()".parse().unwrap()) //TODO actual thing
    }

    async fn log(&self) -> Result<String, Box<dyn Error>> {
        let ssh: SSHAgent = loop {
            match SSHAgent::new(&self.ec2, Path::new("C:/Users/k3nne/Documents/aws/credentials/default/aws-ec2-test.pem")).await {
                Ok(agent) => break agent,
                Err(e) => {
                    // panic!("couldnt make ssh agent! Correct key?");
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            };
        };
        Ok(DEP_exec(&ssh, "cat ~/minecraft/logs/latest.log"))
    }
}