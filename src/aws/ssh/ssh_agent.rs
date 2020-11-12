extern crate ssh2;

use std::net::TcpStream;

use self::ssh2::{Session, Channel};
use std::error::Error;
use crate::aws::virtual_machine::vm::VMNetwork;
use std::path::Path;
use std::io::Read;

pub struct SSHAgent {
    session: Session,
}

impl SSHAgent {

    pub async fn new(vm: &impl VMNetwork, key_path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut ssh_address = match vm.get_public_ip().await {
            Some(ip) => ip,
            None => panic!("tried to get ip but got None, is vm on?") //TODO proper error
        };

        ssh_address.push_str(":22");

        println!("{:?}", ssh_address);

        let tcp = TcpStream::connect(ssh_address)?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        sess.userauth_pubkey_file("ubuntu", None, &key_path, None)?;

        assert!(sess.authenticated());

        Ok(SSHAgent{
            session: sess,
        })
    }

    pub fn execute(&self, command: &str) -> String {
        let mut channel = self.session.channel_session().unwrap();
        let s = Self::execute_with(&mut channel, command);

        channel.wait_close();

        s
    }

    pub fn execute_with(channel: &mut Channel, command: &str) -> String {
        channel.exec(command).unwrap();
        let mut result_string = String::new();
        channel.read_to_string(&mut result_string);

        result_string.push_str(channel.exit_status().unwrap().to_string().as_ref());
        result_string
    }

    pub fn close(&mut self) {
        self.session.disconnect(None, "", None);
    }

    ///Creates session channel to interact with
    pub fn reserve_channel(&self) -> Channel {
        return  self.session.channel_session().unwrap();
    }

}