#![feature(test)]

extern crate bc4py_plotter;
extern crate test;

use bc4py_plotter::cli_tool::*;
use bc4py_plotter::pochash::*;
use std::sync::mpsc::channel;
use workerpool::Pool;
use workerpool::thunk::{Thunk, ThunkWorker};
use std::time::{Instant, Duration};
use colored::Colorize;


fn main() {
    println!("bc4py proof of capacity plotter.");
    let dest = ask_user("destination path?", "./plots");
    let tmp = ask_user("temporary folder?", "./plots");
    let mut address = ask_user("address or node?", "<AddressFormat>");
    if address.len() != 40 {
        let proto = ask_user("node proto?", "http");
        let url = ask_user("node endpoint?", "127.0.0.1:3000");
        let username = ask_user("Auth username?", "user");
        let password = ask_user("Auth password?", "password");
        address = match address_request(&proto, &url, &username, &password) {
            Ok(address) => address,
            Err(err) => panic!("Failed get by {}", err.to_string().bold())
        };
    }

    let mut section_size = 16384;
    let mut section_num = 4;
    loop {
        section_size = ask_user("section size?", &section_size.to_string()).parse().unwrap();
        section_num = ask_user("section number?", &section_num.to_string()).parse().unwrap();
        let total_size = (section_size * section_num) as f32 * (HASH_LENGTH * HASH_LOOP_COUNT) as f32;
        let total_size = total_size / 1000_000_000f32;
        let msg = format!("total size is {} GB, ok?", total_size.to_string().bold());
        let check = ask_user(&msg, "ok");
        if &check == "ok" {
            break;
        } else {
            println!("retry");
        };
    };
    let worker_num: usize = ask_user("how many worker?", "1")
        .parse().expect("worker size is number");
    println!("finish all parameter questions. wait...");

    // throw jobs to worker pool
    let (tx, rx) = channel();
    let pool = Pool::<ThunkWorker<(u32, u32, Result<(), String>)>>::new(worker_num);
    for index in 0..section_num {
        std::thread::sleep(Duration::from_secs(1));
        let start_pos = index * section_size;
        let end_pos = (index + 1) * section_size;
        let address = address.clone();
        let (dest, tmp) = (dest.clone(), tmp.clone());
        pool.execute_to(tx.clone(), Thunk::of( move || {
            let result = plotting(&address, start_pos, end_pos, &tmp, &dest);
            (start_pos, end_pos, result)
        }));
    };

    // waiting for result
    let now = Instant::now();
    println!("\rmsg: success throw {} jobs, waiting...", section_num);
    for (index, (start_pos, end_pos, result)) in rx.iter().enumerate() {
        let index = index as u32 + 1;
        match result {
            Ok(_) => println!("\rmsg: finish {} to {} nonce, {}/{}section, {}minutes passed",
                 start_pos.to_string().bold(), end_pos.to_string().bold(),
                 index, section_num, now.elapsed().as_secs() / 60),
            Err(err_string) => eprintln!("\rError: {}", err_string.bold())
        };
        if index  == section_num {
            break;
        }
    };
    println!(" ");
    println!("finish all jobs, {}minutes", now.elapsed().as_secs() / 60);
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn create_hash(b: &mut Bencher) {
        b.iter(|| generator("NDTTLPOUBQQLC5SZ4BPKK2GK6U3RP6TUKGBCCLDV", 123456));
    }
}