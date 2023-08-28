use winsafe::{self as w, prelude::*, gui};
use crate::password::*;

#[derive(Clone)]
pub struct LoginWindow {
    pub wnd:       gui::WindowMain,
    txt_password: gui::Edit,
    btn_login: gui::Button,
    btn_cancel: gui::Button,
}

impl LoginWindow {
    pub fn new() -> Self {
        let wnd = gui::WindowMain::new( // instantiate the window manager
            gui::WindowMainOpts {
                title: "Login".to_owned(),
                size: (300, 120),
                ..Default::default() // leave all other options as default
            },
        );

        let txt_password = gui::Edit::new(
			&wnd,
			gui::EditOpts {
				position: (20, 30),
				width: 260,
				..Default::default()
			},
		);

        let btn_login = gui::Button::new(
            &wnd, // the window manager is the parent of our button
            gui::ButtonOpts {
                text: "&Login".to_owned(),
                position: (130, 70),
                width: 70,
                ..Default::default()
            },
        );

        let btn_cancel = gui::Button::new(
            &wnd, // the window manager is the parent of our button
            gui::ButtonOpts {
                text: "&Cancel".to_owned(),
                position: (210, 70),
                width: 70,
                ..Default::default()
            },
        );

        let new_self = Self { wnd, txt_password, btn_login, btn_cancel };
        new_self.events(); // attach our events
        new_self
    }

    fn events(&self) {
        let wnd = self.wnd.clone(); // clone so it can be passed into the closure
        self.btn_login.on().bn_clicked(move || {
            // wnd.hwnd().SetWindowText("Hello, world!")?;
            Password::save("test");
            Password::verify("test");
            Ok(())
        });
        self.btn_cancel.on().bn_clicked(move || {
            Ok(())
        })
    }
}