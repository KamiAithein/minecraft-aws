use std::error::Error;
use async_trait::async_trait;

/// Core features of a VM that already exists
#[async_trait]
pub trait VMCore: Sized {
    /// Retrieves virtual_machine.ec2 instance from AWS services by id.
    /// If no instance with matching tag is found then None is returned
    /// Blocks thread until ec2 instance has status <running>
    async fn retrieve(instance_id: &str, role_arn:&str) -> Option<Self>;
    /// Gets current status of this virtual_machine.ec2 instance
    /// returns None if couldn't find instance
    async fn status(&self) -> Option<String>;
    /// Tries to stop this virtual_machine.ec2 instance, returning result string if success
    ///     Errors if cannot
    /// Must not be already off
    async fn stop(&mut self) -> Result<String, Box<dyn Error>>;
    /// Tries to start this virtual_machine.ec2 instance, returning result string if success
    ///     Errors if cannot
    /// Must not be already on
    async fn start(&mut self) -> Result<String, Box<dyn Error>>;
}
#[async_trait]
pub trait VMAdmin: Default {
    /// Creates a new virtual machine
    async fn new() -> Self;
    /// Terminates this virtual machine
    async fn terminate(&mut self) -> Result<String, Box<dyn Error>>;
}
#[async_trait]
pub trait VMNetwork {
    ///returns public ip address of this ec2. Ec2 returns None if ec2 not running
    async fn get_public_ip(&self) -> Option<String>;
}