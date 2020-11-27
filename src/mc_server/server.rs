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
use rocket::futures::TryFutureExt;
use core::fmt;


fn DEP_on_start(ssh: &mut SSHAgent, startup: &Vec<String>) {
    println!("start!");
    if !DEP_exec(ssh, "ls ~/minecraft").contains("server.jar") {
        println!("making minecraft!");
        for command in startup {
            DEP_run(ssh, &**command);
        }
    }
}
fn DEP_on_main(ssh: &SSHAgent, main: &Vec<String>) {

}
fn DEP_exec(ssh: &mut SSHAgent, cmd: &str) -> String {
    return ssh.execute(cmd).unwrap();
}
fn DEP_run(ssh: &mut SSHAgent, cmd: &str) {
    println!("cmd: {}, result: {}", cmd, ssh.execute(cmd).unwrap());
}


///a mc server
#[async_trait]
pub trait Server {
    ///Start the given mc server retuurn ip
    async fn start(&mut self) -> Result<(String), Box<dyn Error>>;
    ///Stop the given mc server
    async fn stop(&mut self) -> Result<(), Box<dyn Error>>;
    ///Execute the mc command given on the server, returning the server response if no error
    async fn command(&mut self, cmd: &str) -> Result<String, Box<dyn Error>>;
    ///Get the current log of the server
    async fn log(&mut self) -> Result<String, Box<dyn Error>>;

    async fn get_ip(&self) -> Result<String, Box<dyn Error>>;

    async fn status(&self) -> State;
}
#[derive(Clone)]
pub enum State {
    STARTING, //ec2 starting
    STARTED,    //mcserver started/ing
    ERROR,
    STOPPING,
    STOPPED
}
impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            State::STARTING => {
                write!(f, "STARTING")
            },
            State::STARTED => {
                write!(f, "STARTED")
            },
            State::ERROR => {
                write!(f, "ERROR")
            },
            State::STOPPING => {
                write!(f, "STOPPING")
            },
            State::STOPPED => {
                write!(f, "STOPPED")
            }
        }
    }
}
pub struct MCServer {
    ec2: Ec2Object,
    config: Ec2Config,
    ssh: SSHAgent,
    state: State
}

impl MCServer {
    pub async fn new(ec2: Ec2Object, config: Ec2Config, ssh: SSHAgent) -> MCServer {
        let status = match ec2.status().await {
            Some(val) => val,
            None => panic!("No status! Correct id?")
        };
        let mut state: State = match &status[..] {
            "stopped" => State::STOPPED,
            "stopping" => State::STOPPED,
            "running" => State::STARTED,
            _ => State::STOPPED
        };

        MCServer {
            ec2,
            config,
            ssh,
            state
        }
    }
}

#[async_trait]
impl Server for MCServer {
    async fn start(&mut self) -> Result<(String), Box<dyn Error>> {
        let status = match self.ec2.status().await {
            Some(val) => val,
            None => panic!("No status! Correct id?")
        };
        match &status[..] {
            "stopped" => {
                self.state = State::STARTING;
                self.ec2.start().await;
            }
            "stopping" => {
                while self.ec2.status().await.is_none() || self.ec2.status().await.unwrap() == "stopping" {}
                self.state = State::STARTING;
                self.ec2.start().await;
            }
            _ => {}
        }


        let init: Vec<String> = self.config.init_script.as_ref().unwrap().clone();
        let main: Vec<String> = self.config.main_script.as_ref().unwrap().clone();

        // thread::spawn(move ||{
        if !DEP_exec(&mut self.ssh, "ls ~/minecraft").contains("server.jar") {
            for command in init {
                DEP_run(&mut self.ssh, &*command);
            }
        }
            if !DEP_exec(&mut self.ssh, "ps -aux").contains("minecraft") {
                for command in main {
                    println!("main: {}", self.ssh.execute(&*command).unwrap());
                    // run(ssh, &**command).await;
                }
            }
        self.state = State::STARTED;

        // });
        // DEP_on_start(&ssh, &init.clone());
        // DEP_on_main(&ssh, &main.clone());

        let ip = self.get_ip().await.unwrap();
        return Ok((ip));//TODO actual return
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error>> {

        self.command("stop").await;
        self.state = State::STOPPED;
        DEP_run(&mut self.ssh, &*format!("sudo shutdown +1"));
        self.ssh.close();
        Ok(()) //TODO actual return
    }

    async fn command(&mut self, cmd: &str) -> Result<String, Box<dyn Error>> {

        DEP_run(&mut self.ssh, &*format!("sudo screen -S mc -X stuff \"{}^M\"", cmd));

        Ok("()".parse().unwrap()) //TODO actual thing
    }

    async fn log(&mut self) -> Result<String, Box<dyn Error>> {
        match self.state {
            State::STARTED => {
                Ok(DEP_exec(&mut self.ssh, "cat ~/minecraft/logs/latest.log").replace("\n", "<br/>"))
            }
            _ => {
                Ok(format!("server is either in a starting or stopping state, state: {}", self.state))
            }
        }
    }

    async fn get_ip(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.ec2.get_public_ip().await.unwrap_or_else(|| "couldnt get ip".parse().unwrap()))
    }
    async fn status(&self) -> State {
        self.state.clone()
    }

}