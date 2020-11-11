// extern crate tokio_test;

pub mod virtual_machine;
pub mod credentials;
pub mod ssh;

#[cfg(test)]
mod tests {
    use crate::aws::virtual_machine::vm::VMCore;
    use std::fs::File;
    use crate::aws::credentials::set_env::set_env_cred_from;
    use crate::aws::virtual_machine::ec2;
    use crate::aws::virtual_machine::ec2::instance::Ec2Object;
    use crate::aws::ssh::ssh_agent::SSHAgent;
    use std::path::Path;

 
    // #[tokio::test]
    // pub async fn test() {
    //     println!("Start!");
    //     let iam_cred_file = File::open("C:/Users/k3nne/Documents/aws/credentials/mc-server/new_user_credentials.csv").unwrap();
    //     (set_env_cred_from(iam_cred_file)).await;
    //     println!("Credentials!");
    //
    //     let mut ec2 = match ec2::instance::Ec2Object::retrieve("i-0005f52626f71c0d9", "arn:aws:iam::417217345236:role/i_am_admin").await {
    //         Some(ec2) => ec2,
    //         None => panic!("Couldn't find instance! Does it exist?")
    //     };
    //     println!("ec2!");
    //     let status = match ec2.status().await {
    //         Some(val) => val,
    //         None => panic!("No status! Correct id?")
    //     };
    //     match &status[..] {
    //         "stopped" => {
    //             ec2.start().await;
    //             let ssh = match SSHAgent::new(&ec2, Path::new("C:/Users/k3nne/Documents/aws/credentials/default/aws-ec2-test.pem")).await {
    //                 Ok(agent) => agent,
    //                 Err(e) => panic!("couldnt make ssh agent! Correct key?")
    //             };
    //             println!("cmd: {}, result: {}", "ls", ssh.execute("ls").await);
    //             ec2.stop();
    //         }
    //         "running" => {
    //             let ssh = match SSHAgent::new(&ec2, Path::new("C:/Users/k3nne/Documents/aws/credentials/default/aws-ec2-test.pem")).await {
    //                 Ok(agent) => agent,
    //                 Err(e) => panic!("couldnt make ssh agent! Correct key?")
    //             };
    //             println!("cmd: {}, result: {}", "ls", ssh.execute("ls").await);
    //             ec2.stop();
    //         }
    //         _ => {}
    //     }
    //     println!("status: {:?}", (ec2.status()).await);
    //
    // }
}
