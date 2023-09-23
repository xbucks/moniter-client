extern crate bcrypt;

use std::fs::OpenOptions;
use std::io::Write;
use std::io::Read;
use std::str;
use bcrypt::{DEFAULT_COST, hash, verify};

#[derive(Debug, Clone, Copy)]
pub struct Password {
    pub save: fn(),
    pub verify: fn(),
}

impl Password {
    pub fn save(pass: &str) {
        let hashed = hash(pass, DEFAULT_COST).unwrap();
        let mut fileRef = OpenOptions::new()
            .write(true)
            .create(true)
            .open(".sys/log1.txt")
            .expect("Unable to open log1 file to log.");

        fileRef.write(hashed.as_bytes()).expect("write failed");
    }

    pub fn verify(pass: &str) -> bool {
        let mut fileRef = OpenOptions::new()
            .read(true)
            .open(".sys/log1.txt")
            .expect("Unable to open log1 file");

        let mut data = String::new();
        fileRef.read_to_string(&mut data);
        let valid = verify(pass, &data).unwrap();
        valid
    }
}