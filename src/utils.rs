use bech32::Bech32;
use std::str::FromStr;


pub fn addr2ver_identifier(address: &str) -> Result<Vec<u8>, String> {
     match Bech32::from_str(address) {
        Ok(bech32) => Ok(bech32.data().into_iter().map(|p|p.to_u8()).collect()),
        Err(err) => Err(format!("Error: failed get bech32 \"{}\"", err.to_string()))
    }
}
