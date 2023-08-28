use winsafe::{self as w, prelude::*, gui};
use winsafe::co::ES;
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
        let wnd = gui::WindowMain::new(
            gui::WindowMainOpts {
                title: "Login".to_owned(),
                size: (300, 120),
                ..Default::default()
            },
        );

        let txt_password = gui::Edit::new(
			&wnd,
			gui::EditOpts {
				position: (20, 30),
				width: 260,
                edit_style: ES::PASSWORD,
				..Default::default()
			},
		);

        let btn_login = gui::Button::new(
            &wnd,
            gui::ButtonOpts {
                text: "&Login".to_owned(),
                position: (130, 70),
                width: 70,
                ..Default::default()
            },
        );

        let btn_cancel = gui::Button::new(
            &wnd,
            gui::ButtonOpts {
                text: "&Cancel".to_owned(),
                position: (210, 70),
                width: 70,
                ..Default::default()
            },
        );

        let new_self = Self { wnd, txt_password, btn_login, btn_cancel };
        new_self.events();
        new_self
    }

    fn events(&self) {
        let wnd = self.wnd.clone();

        let self2 = self.clone();
        self.txt_password.on().en_change(move || {
			// let text = self2.txt_name.text();
			// self2.wnd.set_text(&text);
			Ok(())
		});

        let self2 = self.clone();
        self.btn_login.on().bn_clicked(move || {
            let text = self2.txt_password.text();
            Password::save(&text);
            Password::verify(&text);
            println!("{}", text);
            Ok(())
        });

        self.btn_cancel.on().bn_clicked(move || {
            wnd.hwnd().DestroyWindow().ok();
            Ok(())
        })
    }
}