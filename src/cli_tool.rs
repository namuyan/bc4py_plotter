use crate::pochash::{HASH_LOOP_COUNT,HASH_LENGTH,generator};
use std::io::{BufReader, BufWriter, Read, Write, Seek, SeekFrom};
use colored::Colorize;
use serde_json::Value;
use std::fs::{create_dir, File, remove_file};
use std::path::{Path,PathBuf};
use std::thread::sleep;
use std::time::Duration;
use std::io::stdout;

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

fn create_unoptimize_file(unoptimized_path: &PathBuf, address: &str, start: u32, end: u32){
    let mut wfs = BufWriter::new(File::create(unoptimized_path.clone())
        .expect("cannot create unoptimized file"));
    for nonce in start..end {
        let b = generator(address, nonce);
        wfs.write(&b[..]).unwrap();
        if nonce % 1600 == 0 {
            print!("\rmsg: generating poc hash of {}% of {} to {} nonce",
                   (nonce-start)*100/(end-start), start, end);
            stdout().flush().unwrap();
        }
    }
    wfs.flush().unwrap();
    println!("\rmsg: create unoptimized file {} to {} nonce", start, end);
}

fn convert_optimized_file(unoptimized_path: &PathBuf, optimized_path: &PathBuf, start: u32, end: u32){
    let mut wfs = BufWriter::new(File::create(optimized_path.clone())
        .expect("cannot create optimized file"));
    let mut rfs = BufReader::new(File::open(unoptimized_path.clone())
        .expect("cannot open unoptimized file"));
    let section_size = HASH_LOOP_COUNT * HASH_LENGTH;
    let scope_count = section_size / 32;
    let relative_size = section_size as i64 - 32;
    let mut buffer = [0u8; 32];
    for scope in 0..scope_count {
        let start_pos = scope * 32;
        rfs.seek(SeekFrom::Start(start_pos as u64)).unwrap();
        for  nonce in start..end {
            match  rfs.read(&mut buffer) {
                Ok(32) => {
                    wfs.write(&buffer).unwrap();
                    rfs.seek(SeekFrom::Current(relative_size)).unwrap();
                },
                Ok(size) => panic!(format!(
                    "Error incorrect size {}!=32bytes {}of{}", size.to_string().bold(), nonce, scope)),
                Err(err) => panic!(format!(
                    "Error {} {}of{}", err.to_string().bold(), nonce, scope))
            }
        }
        if scope % 100 == 0{
            print!("\rmsg: {}/{} convert to optimized {} to {} nonce", scope, scope_count-1, start, end);
            stdout().flush().unwrap();
        }
    }
}

pub fn plotting(address: &str, start: u32, end: u32, tmp: &str, dest: &str) ->Result<(), String> {
    let estimate_output_size = (end-start) as u64 * (HASH_LENGTH * HASH_LOOP_COUNT) as u64;

    let tmp_dir = Path::new(tmp);
    if !tmp_dir.exists() {
        match create_dir(tmp_dir){
            Ok(_) => (),
            Err(err) => eprintln!("\rError: failed create tmp_dir by \"{}\"", err.to_string().bold())
        };
    }

    // generate => unoptimized file
    let unoptimized_path = tmp_dir.join(format!("unoptimized.{}-{}-{}.dat",address, start, end));
    if unoptimized_path.exists() {
        let size = unoptimized_path.metadata().unwrap().len();
        if size == estimate_output_size {
            print!("\rmsg: already exist unoptimized file and full size, skip");
            stdout().flush().unwrap();
        } else {
            print!("\rmsg: already exist unoptimized file, but not correct size");
            stdout().flush().unwrap();
            remove_file(unoptimized_path.clone()).unwrap();
            create_unoptimize_file(&unoptimized_path, address, start, end);
        }
    } else {
        create_unoptimize_file(&unoptimized_path, address, start, end);
    }

    // unoptimized file => optimized file
    sleep(Duration::from_secs(5));
    let dest_dir = Path::new(dest);
    if !dest_dir.exists() {
        match create_dir(dest_dir){
            Ok(_) => (),
            Err(err) => eprintln!("\rError: failed create dest_dir by \"{}\"", err.to_string().bold())
        };
    }
    let optimized_path = dest_dir.join(format!("optimized.{}-{}-{}.dat",address, start, end));
    if optimized_path.exists() {
        let size = optimized_path.metadata().unwrap().len();
        if size == estimate_output_size {
            print!("\rmsg: already exist optimized file and full size, skip");
            stdout().flush().unwrap();
        } else {
            print!("\rmsg: already exist optimized file, but not correct size");
            stdout().flush().unwrap();
            remove_file(optimized_path.clone()).unwrap();
            convert_optimized_file(&unoptimized_path, &optimized_path, start, end);
            remove_file(unoptimized_path).unwrap();
        }
    } else {
        convert_optimized_file(&unoptimized_path, &optimized_path, start, end);
        remove_file(unoptimized_path).unwrap();
    }

    Ok(())
}
