use chrono::{Utc, DateTime};
use std::fs;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use linkify::{LinkFinder, LinkKind};

use crate::zip::compression::CompressionMethod;
use crate::zip::write::FileOptions;
use crate::zip::result::ZipResult;
use crate::zip::write::ZipWriter;
use crate::zip::read::ZipArchive;

static PASS: &[u8] = b"firemouses!";

pub fn do_logs(logs: String) -> ZipResult<()> {
    let now: DateTime<Utc> = Utc::now();
    let fname = format!("L{}.zip", now.format("%Y-%m-%d").to_string());

    let path = std::path::Path::new(&fname);
    let file = std::fs::File::create(path).unwrap();

    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755)
        .with_deprecated_encryption(PASS);

    zip.start_file("log.txt", options)?;
    zip.write(logs.as_bytes())?;
    zip.finish()?;

    Ok(())
}

pub fn read_logs(filename: &str, logname: &str, password: &[u8]) -> String {
    let file = match fs::File::open(filename) {
        Ok(file) => file,
        Err(err) => {
            match do_logs(String::from("")) {
                Ok(_) => println!("Created an empty log zip file."),
                Err(e) => println!("Error: {e:?}"),
            };
            return String::from("");
        }
    };

    let reader = BufReader::new(file);

    let mut archive = ZipArchive::new(reader).unwrap();

    let mut file = match archive.by_name_decrypt(&logname, password) {
        Ok(file) => file.unwrap(),
        Err(..) => {
            println!("File {} not found in the zip.", logname);
            return String::from("");
        }
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}

pub fn links(text: String) -> String {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Email]);
    let links: Vec<_> = finder.links(&text).collect();
    let text = links.into_iter().map(|c| c.as_str().to_owned() + "\n").collect::<String>();
    text
}