extern crate reqwest;
extern crate serde_json;

use sha2::{Sha512, Digest};
use std::mem::transmute;
use std::io::{BufReader, BufWriter, Read, Write, Seek, SeekFrom};
use colored::Colorize;
use serde_json::Value;
use std::fs::{create_dir, File, rename, remove_file};
use std::path::Path;
use std::sync::{Mutex, Arc};

pub const HASH_LOOP_COUNT: usize = 512;
pub const HASH_LENGTH: usize = 64;
pub const SEED_LENGTH: usize = 44;


#[inline]
fn slice_replace(src: &mut [u8], by: &[u8]){
    debug_assert_eq!(src.len(), by.len());
    for (a, &b) in src.iter_mut().zip(by.iter()) {
        *a = b;
    }
}

pub fn generator(address: &str, nonce: u32) ->[u8;HASH_LOOP_COUNT*HASH_LENGTH] {
    let mut source = [0u8; HASH_LOOP_COUNT * HASH_LENGTH + SEED_LENGTH];
    let total_length = SEED_LENGTH + HASH_LOOP_COUNT * HASH_LENGTH;
    debug_assert_eq!(address.as_bytes().len() + 4, SEED_LENGTH);

    // seed ..-[address 40bytes]-[nonce 4bytes]
    let bytes: [u8; 4] = unsafe { transmute(nonce.to_le()) };
    slice_replace(&mut source[(total_length-4)..], &bytes);
    slice_replace(&mut source[(total_length-SEED_LENGTH)..(total_length-4)], address.as_bytes());
    //println!("source={:?}", &source[(total_length-SEED_LENGTH)..]);

    // seed [hash(HASH_LENGTH)]-...-[hash0]-[address 40bytes]-[nonce 4bytes]
    // [hashN] = SHA512([hash(N-1)]-...-[hash0]-[address 40bytes]-[nonce 4bytes])
    let start_index = total_length - SEED_LENGTH;
    let mut final_hash = [0u8; HASH_LENGTH];
    for index in 0..(HASH_LOOP_COUNT+1) {
        let pos = start_index - index * HASH_LENGTH;
        let hash = Sha512::digest(&source[pos..]);
        if pos == 0 {
            slice_replace(&mut final_hash, &hash);
        } else {
            slice_replace(&mut source[(pos-HASH_LENGTH)..pos], &hash);
        }
    }
    //println!("final={:?}\nsource={:?}", final_hash, &source[..]);

    // all hash_ints XOR with final_int
    // from: [hash(HASH_LENGTH)]-...-[hash0]-[address 40bytes]-[nonce 4bytes]
    // to  : [hash'0]- ... - [hash'(HASH_LENGTH)]
    let mut output = [0u8; HASH_LOOP_COUNT * HASH_LENGTH];
    for (index, item) in output.iter_mut().enumerate() {
        let inner_pos = index % HASH_LENGTH;  // 0~31
        let outer_pos = index / HASH_LENGTH;
        let x = &final_hash[inner_pos];
        let y = &source[(HASH_LOOP_COUNT-outer_pos-1)*HASH_LENGTH+inner_pos];
        *item = x ^ y;
        //println!("{} {:?}=={:?}^{:?}", index, item, x, y);
    }
    //println!("output={:?}", &output[..]);
    output
}

pub fn ask_user(question: &str, default: &str) ->String {
    print!("Q. {} default=\"{}\" >> ", question.underline(), default.bold());
    std::io::stdout().flush().unwrap();
    loop {
        let mut s = String::new();
        match std::io::stdin().read_line(&mut s) {
            Ok(_) => {
                let s: String = s.trim().parse().ok().unwrap();
                if s.is_empty() {
                    return default.to_owned();
                } else {
                    return s;
                }
            },
            Err(err) => println!("{}: \"{}\" retry.", "Error".red() , err.to_string().bold())
        }
    }
}

pub fn address_request(proto: &str, url: &str, username: &str, password: &str) -> Result<String, String> {
    println!("Request node new address...");
    let client = reqwest::Client::new();
    let url = format!("{}://{}/private/newaddress?account=@Mining", proto, url);
    let mut response = match client.get(&url)
        .basic_auth(username.to_owned(), Some(password.to_owned()))
        .send(){
            Ok(res) => res,
            Err(err) => return Err(err.to_string())
    };
    let code = response.status().as_u16();
    if code != 200 {
        let res = response.text().unwrap();
        return Err(format!("bad status code {} {}", code,  res.bold()));
    }
    let res: Value = match response.json() {
        Ok(res) => res,
        Err(err) => return Err(err.to_string())
    };
    let address = match res.get("address") {
        Some(address) => address.as_str().unwrap(),
        None => return Err(String::from("Not found address key"))
    };
    println!("Success get address \"{}\"", address.bold());
    Ok(address.to_owned())
}

pub fn plotting(address: &str, start: u32, end: u32, tmp: &str, dest: &str, lock: Arc<Mutex<u32>>) ->Result<(), String> {
    let tmp_dir = Path::new(tmp);
    if !tmp_dir.exists() {
        match create_dir(tmp_dir){
            Ok(_) => (),
            Err(err) => eprintln!("Error: failed create tmp_dir by \"{}\"", err.to_string().bold())
        };
    }

    // generate => unoptimized file
    let unoptimized_path = tmp_dir.join(format!("unoptimized.{}-{}-{}.dat",address, start, end));
    {
        let mut wfs = BufWriter::new(File::create(unoptimized_path.clone())
            .expect("cannot create unoptimized file"));
        for nonce in start..end {
            let b = generator(address, nonce);
            wfs.write(&b).unwrap();
        }
        wfs.flush().unwrap();
    }

    // unoptimized file => optimized file
    let optimized_path = tmp_dir.join(format!("optimized.{}-{}-{}.dat",address, start, end));
    {
        let mut wfs = BufWriter::new(File::create(optimized_path.clone())
            .expect("cannot create optimized file"));
        let mut rfs = BufReader::new(File::open(unoptimized_path.clone())
            .expect("cannot open unoptimized file"));
        let section_size = HASH_LOOP_COUNT * HASH_LENGTH;
        let section_count = section_size / 32;
        let relative_size = section_size as i64 - 32;
        let mut buffer = [0u8; 32];
        for section in 0..section_count {
            let start_pos = section * 32;
            rfs.seek(SeekFrom::Start(start_pos as u64)).unwrap();
            for index in start..end {
                match  rfs.read(&mut buffer) {
                    Ok(32) => {
                        wfs.write(&buffer).unwrap();
                        rfs.seek(SeekFrom::Current(relative_size)).unwrap();
                    },
                    Ok(size) => panic!(format!(
                        "Error incorrect size {}!=32bytes {}of{}", size.to_string().bold(), index, section)),
                    Err(err) => panic!(format!(
                        "Error {} {}of{}", err.to_string().bold(), index, section))
                }
            }
            wfs.flush().unwrap();
        }
    }
    remove_file(unoptimized_path).unwrap();

    // copy optimized file => destination path
    let dest_dir = Path::new(dest);
    if !dest_dir.exists() {
        match create_dir(dest_dir){
            Ok(_) => (),
            Err(err) => eprintln!("Error: failed create dest_dir by \"{}\"", err.to_string().bold())
        };
    }
    let dest_path = dest_dir.join(format!("optimized.{}-{}-{}.dat",address, start, end));
    let mut n = lock.lock().unwrap();
    *n += 1;
    rename(optimized_path.clone(), dest_path).unwrap();
    Ok(())
}
