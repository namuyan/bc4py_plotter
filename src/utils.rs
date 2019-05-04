use bech32::{Bech32,convert_bits};
use std::str::FromStr;


#[inline]
pub fn addr2ver_identifier(address: &str) -> Result<Vec<u8>, String> {
    // return [ver+identifier] bytes
    let (ver, mut identifier) = match addr2params(address) {
        Ok((_, ver, identifier)) => (ver, identifier),
        Err(err) => return Err(err.to_string())
    };
    identifier.insert(0, ver);
    Ok(identifier)
}

#[inline]
pub fn params2bech(hrp: &str, ver: u8, identifier:&[u8]) -> Result<Bech32, bech32::Error> {
    let mut data = convert_bits(identifier, 8, 5, true)?;
    data.insert(0, ver);
    Bech32::new_check_data(hrp.to_string(), data)
}

#[inline]
pub fn addr2params(addr: &str) -> Result<(String, u8, Vec<u8>), bech32::Error> {
    // return (hrp, version, identifier)
    let bech = Bech32::from_str(addr)?;
    let ver = match bech.data().get(0) {
        Some(ver) => ver.to_owned().to_u8(),
        None => return Err(bech32::Error::InvalidLength)
    };
    let identifier = convert_bits(&bech.data()[1..], 5, 8, false)?;
    Ok((bech.hrp().to_string(), ver, identifier))
}
