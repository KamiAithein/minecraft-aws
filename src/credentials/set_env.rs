extern crate rusoto_credential;
extern crate tokio;

use std::env;
use crate::credentials::credential::Credential;
use std::fs::File;
use crate::credentials;

const ACCESS_KEY_ID: &str = "AWS_ACCESS_KEY_ID";
const SECRET_ACCESS_KEY: &str = "AWS_SECRET_ACCESS_KEY";
const SESSION_TOKEN: &str = "AWS_SESSION_TOKEN";

pub async fn set_env_cred(cred: Credential) -> () {
    env::set_var(ACCESS_KEY_ID, cred.access_key_id);
    env::set_var(SECRET_ACCESS_KEY, cred.secret_access_key);
}

pub async fn set_env_cred_from(csv: File) {
    let creds = credentials::credential::Credential::new_with_csv(&(csv)).unwrap();
    credentials::set_env::set_env_cred(
        creds
    ).await;
}

#[cfg(test)]
mod tests{
    use super::*;
    use rusoto_credential::{EnvironmentProvider, ProvideAwsCredentials};

    #[test]
    fn var_set(){
        const ID: &str = "id";
        const KEY: &str = "key";
        const TOKEN: &str = "token";
        tokio_test::block_on(set_env_cred(Credential::new(ID, KEY)));

        let creds = tokio_test::block_on(EnvironmentProvider::default().credentials()).unwrap();

        assert_eq!(creds.aws_access_key_id(), ID);
        assert_eq!(creds.aws_secret_access_key(), KEY);
    }
}