extern crate ssh2;

use std::fs::File;

use rust_ec2::credentials::set_env::set_env_cred_from;
use rust_ec2::virtual_machine::ec2;
use rust_ec2::virtual_machine::vm::{VMCore, VMNetwork};

use std::net::TcpStream;
use std::path::Path;
use ssh2::Session;
use std::io::Read;

#[tokio::main]
async fn main() {
    let iam_cred_file = File::open("C:/Users/k3nne/Documents/aws/credentials/mc-server/new_user_credentials.csv").unwrap();
    set_env_cred_from(iam_cred_file).await;

    let mut ec2 = ec2::instance::Ec2Object::retrieve().await.unwrap();
    ec2.status().await;
    ec2.start().await;

    //refactor this into rust_ec2 library
    let mut ssh_address = ec2.get_pub_ip().await.unwrap();
    ssh_address.push_str(":22");

    let tcp = TcpStream::connect(ssh_address).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    //TODO needs to authenticate
    sess.userauth_pubkey_file("ubuntu", None, Path::new("C:/Users/k3nne/Documents/aws/credentials/default/aws-ec2-test.pem"), None).unwrap();
    assert!(sess.authenticated());

    let mut channel = sess.channel_session().unwrap();
    channel.exec("touch if_this_works_im_gonna_cream_omg").unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    println!("{}", s);
    channel.wait_close();
    println!("{}", channel.exit_status().unwrap());

    // ec2.stop().await;
}
