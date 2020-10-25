extern crate csv;

use std::fs::File;
use std::error::Error;
use self::csv::StringRecord;

/// In order array of headers in credentials file
const HEADERS: &'static [&'static str] = &["User name","Password","Access key ID","Secret access key","Console login link"];

/// One single credential needed to access AWS
pub struct Credential {
    pub access_key_id: String,
    pub secret_access_key: String
}

/// Gets a credential from a StringRecord previously retrieved from a credential.csv
/// Must have all values included in HEADERS present otherwise will be considered malformed
fn cred_from_str_rec(rec: &StringRecord) -> Result<Credential, Box<dyn Error>> {
    assert_eq!(rec.len(), HEADERS.len());

    let rec_vec: Vec<&str> = rec.into_iter().collect();

    let find_val_pos = |key: &str| -> usize {
        match HEADERS.iter().position(|header| header == &key) {
            Some(val) => val,
            None => panic!(format!("could not find: <{}> in <{:?}>", key, HEADERS))
        }
    };
    let cred = Credential{
        access_key_id: rec_vec[find_val_pos(&"Access key ID")].to_string(),
        secret_access_key: rec_vec[find_val_pos(&"Secret access key")].to_string()
    };
    Ok(cred)
}

impl Credential {
    pub fn new(key_id: &str, secret_access_key: &str) -> Credential {
        return Credential {
            access_key_id:key_id.to_string(),
            secret_access_key:secret_access_key.to_string()
        }
    }
    pub fn new_with_csv(csv: &File) -> Result<Credential, Box<dyn Error>> {
        let mut reader = csv::Reader::from_reader(csv);

        for result in reader.records() { //returns first
            let record = result?;
            let cred = cred_from_str_rec(&record);
            return cred;
        }
        panic!("Expected value in csv, found none!");
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn correct_val_new(){
        const ID: &str = "id";
        const SECRET: &str = "secret";

        let cred = Credential::new(ID, SECRET);
        assert_eq!(cred.secret_access_key, SECRET);
        assert_eq!(cred.access_key_id, ID);
    }
    #[test]
    fn correct_val_new_with_csv() -> Result<(), Box<dyn Error>> {
        let file = File::open("data/test_user_credentials_valid")?;
        let cred = Credential::new_with_csv(&file)?;
        assert_eq!(cred.access_key_id, "id");
        assert_eq!(cred.secret_access_key, "key");
        Ok(())
    }

    #[test]
    fn error_new_with_csv() -> Result<(), Box<dyn Error>> {
        let file = File::open("data/test_user_credentials_invalid")?;
        let cred = Credential::new_with_csv(&file);
        match cred {
            Ok(_val) => panic!("gave valid credential from invalid csv"),
            Err(_e) => {}
        }
        Ok(())
    }
}
