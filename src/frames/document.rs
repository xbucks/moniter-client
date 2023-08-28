use winsafe::{self as w, prelude::*, gui};
use winsafe::co::ES;
use crate::password::*;

#[derive(Clone)]
pub struct DocumentWindow {
    pub wnd:       gui::WindowMain,
    txt_content: gui::Edit,
    txt_password: gui::Edit,
    btn_load: gui::Button,
}

impl DocumentWindow {
    pub fn new() -> Self {
        let wnd = gui::WindowMain::new(
            gui::WindowMainOpts {
                title: "Document".to_owned(),
                size: (800, 600),
                ..Default::default()
            },
        );

        let txt_content = gui::Edit::new(
			&wnd,
			gui::EditOpts {
				position: (20, 20),
				width: 560,
                height: 400,
                edit_style: ES::PASSWORD,
				..Default::default()
			},
		);

        let txt_password = gui::Edit::new(
			&wnd,
			gui::EditOpts {
				position: (500, 550),
				width: 120,
                edit_style: ES::PASSWORD,
				..Default::default()
			},
		);

        let btn_load = gui::Button::new(
            &wnd,
            gui::ButtonOpts {
                text: "&Load".to_owned(),
                position: (700, 550),
                ..Default::default()
            },
        );

        let new_self = Self { wnd, btn_load, txt_content, txt_password };
        new_self.events();
        new_self
    }

    fn events(&self) {
        let wnd = self.wnd.clone();

        let self2 = self.clone();
        self.btn_load.on().bn_clicked(move || {
            let text = self2.txt_password.text();
            let valid: bool = Password::verify(&text);
            Ok(())
        });
    }
}