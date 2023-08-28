use chrono::{Utc, DateTime};
use std::fs;
use std::io::{self, BufRead, Read};
use std::io::BufReader;

use crate::zip::read::ZipArchive;

pub fn read_logs(filename: &str, logname: &str, password: &[u8]) -> String {
    let lname = format!("text/{}", logname);

    let now: DateTime<Utc> = Utc::now();
    let file = match fs::File::open(filename) {
        Ok(file) => file,
        Err(err) => panic!("There was a problem opening the file: {:?}", err)
    };

    let reader = BufReader::new(file);

    let mut archive = ZipArchive::new(reader).unwrap();

    let mut file = match archive.by_name_decrypt(&lname, password) {
        Ok(file) => file.unwrap(),
        Err(..) => {
            println!("File text/{} not found in the zip.", logname);
            return String::from("");
        }
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}