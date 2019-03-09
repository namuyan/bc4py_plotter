use crate::pochash::{HASH_LOOP_COUNT,HASH_LENGTH};
use crate::cli_tool::ask_user;
use std::fs::{read_dir, rename, File};
use std::io::{stdout, BufReader, BufWriter, Read, Write, Seek, SeekFrom};
use std::path::Path;
use std::time::Instant;
use regex::Regex;


pub fn plot_joiner(address: &str, length: usize, start: usize, tmp: &str, dest: &str) -> Result<(), String> {
    let now = Instant::now();
    let dir = read_dir(tmp).map_err(|err| err.to_string())?;
    let re = Regex::new("^optimized\\.([A-Z0-9]{40})\\-([0-9]+)\\-([0-9]+)\\.dat$").unwrap();

    let mut files = vec![];
    for path in dir {
        let path = path.map_err(|err| err.to_string())?;
        let name = path.file_name().to_str().unwrap().to_string();
        let meta = path.metadata().map_err(|err| err.to_string())?;
        let path = path.path();
        match re.captures(&name) {
            Some(c) => {
                if c.len() != 4 {continue}
                let check_address = c.get(1).unwrap().as_str().to_owned();
                if check_address != address {continue}
                let start: usize = c.get(2).unwrap().as_str().parse().unwrap();
                let end: usize = c.get(3).unwrap().as_str().parse().unwrap();
                if end - start != length {continue}
                let estimate_size = length * HASH_LOOP_COUNT * HASH_LENGTH;
                if estimate_size as u64 != meta.len() {continue}
                files.push((path, start, end))
            },
            None => {
                println!("msg: ignore file {}", name)
            }
        }
    }

    // reorder
    let mut join_order = vec![];
    let mut tmp_index = start;
    while files.len() > 0 {
        let mut count = 0;
        for (index, (path, start, end)) in files.clone().iter().enumerate(){
            if *start == tmp_index {
                join_order.push(path.clone());
                tmp_index = *end;
                files.remove(index);
                count += 1;
                break;
            }
        }
        if count == 0 {
            return Err(format!("there is {} unrelated files", files.len()));
        }
    }


    // ask user
    println!("msg: Let's join with {} files", join_order.len());
    for (index, path) in join_order.iter().enumerate() {
        println!("msg: {} {:?}", index, path.file_name().unwrap());
    }
    let a = ask_user("ok?", "ok");
    if a != "ok" {return Err("user stopped".to_owned());}

    // let's join
    let work_file = format!("optimized.{}-{}-{}.tmp", address, start, start+length*join_order.len());
    let work_file = Path::new(dest).join(work_file);
    let mut wfs = BufWriter::new(File::create(work_file.clone()).unwrap());
    let scope_number = HASH_LOOP_COUNT * HASH_LENGTH / 32;
    let mut buffer = vec![0u8;length*32].into_boxed_slice();
    for scope in 0..scope_number {
        let start_pos = (length * 32 * scope) as u64;
        for fs in join_order.iter(){
            let mut rfs = BufReader::new(File::open(fs).unwrap());
            rfs.seek(SeekFrom::Start(start_pos)).unwrap();
            let size = rfs.read(&mut buffer).unwrap();
            assert_eq!(size, length*32);
            wfs.write(&buffer).unwrap();
        }
        print!("\r{}/{} finish copy scope, {}Min passed",
                 scope + 1, scope_number, now.elapsed().as_secs()/60);
        stdout().flush().unwrap();
    }

    let output = format!("optimized.{}-{}-{}.dat", address, start, start+length*join_order.len());
    let output = Path::new(dest).join(output);
    rename(work_file, output).unwrap();
    Ok(())
}