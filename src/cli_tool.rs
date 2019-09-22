use crate::pochash::{HASH_LOOP_COUNT,HASH_LENGTH,generator};
use crate::utils::*;
use std::io::{BufReader, BufWriter, Read, Write, Seek, SeekFrom};
use colored::Colorize;
use serde_json::Value;
use std::fs::{create_dir, File, remove_file, rename};
use std::path::{Path,PathBuf};
use std::thread::sleep;
use std::time::{Instant, Duration};
use std::io::stdout;

const MAX_MEMORY_SIZE: usize = 1500; // MB

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

pub fn address_request(url: &str, username: &str, password: &str) -> Result<String, String> {
    println!("Request node new address...");
    let client = reqwest::Client::new();
    let mut response = match client.get(url)
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

fn create_unoptimize_file(unoptimized_path: &PathBuf, ver_identifier: &[u8], start: u32, end: u32){
    let mut wfs = BufWriter::new(File::create(unoptimized_path.clone())
        .expect("cannot create unoptimized file"));
    let mut output =  Box::new([0u8;HASH_LOOP_COUNT*HASH_LENGTH]);
    for nonce in start..end {
        generator(ver_identifier, nonce, &mut output);
        wfs.write(&output.to_vec()).unwrap();
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
    let mut split_number= HASH_LOOP_COUNT * HASH_LENGTH / 32;
    loop {
        let memory_size = (end - start) as usize * 32 * split_number / 1_000_000;  // M bytes
        if memory_size < MAX_MEMORY_SIZE {
            println!("\rmsg: split to {}, use {}MB memory", split_number, memory_size);
            break
        }else {
            split_number /= 2;
        }
    }

    let now = Instant::now();
    let section_size = HASH_LOOP_COUNT * HASH_LENGTH;
    let block_count = section_size / split_number / 32;
    let relative_size = (section_size - split_number * 32) as i64;
    let mut buffer = [0u8;32];
    let mut big_buffer = vec![];
    for _ in 0..split_number {
        big_buffer.push(vec![]);
    }
    for block_num in 0..block_count {
        let start_pos = (block_num * split_number * 32) as u64;
        rfs.seek(SeekFrom::Start(start_pos)).unwrap();
        for  nonce in start..end {
            for tmp in big_buffer.iter_mut() {
                match  rfs.read(&mut buffer) {
                    Ok(32) => {
                        tmp.extend_from_slice(&buffer);
                    },
                    Ok(size) => panic!(format!(
                        "Error incorrect size {}!=32bytes {}of{}", size.to_string().bold(), nonce, block_num)),
                    Err(err) => panic!(format!(
                        "Error {} {}of{}", err.to_string().bold(), nonce, block_num))
                }
            }
            rfs.seek(SeekFrom::Current(relative_size)).unwrap();
        }

        for tmp in big_buffer.iter_mut() {
            wfs.write(tmp.as_slice()).unwrap();
            tmp.clear();
        }

        print!("\rmsg: {}/{} convert to optimized {} to {} nonce, {}m passed",
               block_num+1, block_count, start, end, now.elapsed().as_secs()/60);
        stdout().flush().unwrap();
    }
}

pub fn plotting(address: &str, start: u32, end: u32, tmp: &str, dest: &str) ->Result<(), String> {
    let estimate_output_size = (end-start) as u64 * (HASH_LENGTH * HASH_LOOP_COUNT) as u64;

    // folder check
    let tmp_dir = Path::new(tmp);
    let dest_dir = Path::new(dest);
    if !tmp_dir.exists() {
        match create_dir(tmp_dir){
            Ok(_) => (),
            Err(err) => eprintln!("\rError: failed create tmp_dir by \"{}\"", err.to_string().bold())
        };
    }
    if !dest_dir.exists() {
        match create_dir(dest_dir){
            Ok(_) => (),
            Err(err) => eprintln!("\rError: failed create dest_dir by \"{}\"", err.to_string().bold())
        };
    }
    let unoptimized_path = tmp_dir.join(format!("unoptimized.{}-{}-{}.tmp",address, start, end));
    let optimized_path = dest_dir.join(format!("optimized.{}-{}-{}.tmp",address, start, end));
    let output_path = dest_dir.join(format!("optimized.{}-{}-{}.dat",address, start, end));
    if output_path.exists() {
        print!("\rmsg: already exist output file, skip");
        stdout().flush().unwrap();
        return Ok(());
    }

    // get ver_identifier
    let ver_identifier = addr2ver_identifier(address)?;

    // generate => unoptimized file
    if unoptimized_path.exists() {
        let size = unoptimized_path.metadata().unwrap().len();
        if size == estimate_output_size {
            print!("\rmsg: already exist unoptimized file and full size, skip");
            stdout().flush().unwrap();
        } else {
            print!("\rmsg: already exist unoptimized file, but not correct size");
            stdout().flush().unwrap();
            remove_file(unoptimized_path.clone()).unwrap();
            create_unoptimize_file(&unoptimized_path, &ver_identifier, start, end);
        }
    } else {
        create_unoptimize_file(&unoptimized_path, &ver_identifier, start, end);
    }

    sleep(Duration::from_secs(5));

    // unoptimized file => optimized file
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

    // rename to output file
    rename(optimized_path, output_path).unwrap();

    Ok(())
}
