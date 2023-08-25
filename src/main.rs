use core::mem::MaybeUninit;
use winapi::um::winuser;
use std::time::Instant;
use std::io::prelude::*;
use monitor::*;

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
                zip_main();
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
    println!("My activity {:?}", event);
}

fn zip_main() -> i32 {
    let filename = "test.zip";
    match dozip(filename) {
        Ok(_) => println!("File written to {filename}"),
        Err(e) => println!("Error: {e:?}"),
    }

    0
}

fn dozip(filename: &str) -> ZipResult<()> {
    let path = std::path::Path::new(filename);
    let file = std::fs::File::create(path).unwrap();

    let mut zip = ZipWriter::new(file);

    zip.add_directory("test/", Default::default())?;

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755)
        .with_deprecated_encryption(b"test");
    zip.start_file("test/☃.txt", options)?;
    zip.write_all(b"Hello, World!\n")?;

    zip.start_file("test/lorem_ipsum.txt", Default::default())?;
    zip.write_all(LOREM_IPSUM)?;

    zip.finish()?;
    Ok(())
}

const LOREM_IPSUM : &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. In tellus elit, tristique vitae mattis egestas, ultricies vitae risus. Quisque sit amet quam ut urna aliquet
molestie. Proin blandit ornare dui, a tempor nisl accumsan in. Praesent a consequat felis. Morbi metus diam, auctor in auctor vel, feugiat id odio. Curabitur ex ex,
dictum quis auctor quis, suscipit id lorem. Aliquam vestibulum dolor nec enim vehicula, porta tristique augue tincidunt. Vivamus ut gravida est. Sed pellentesque, dolor
vitae tristique consectetur, neque lectus pulvinar dui, sed feugiat purus diam id lectus. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per
inceptos himenaeos. Maecenas feugiat velit in ex ultrices scelerisque id id neque.

Phasellus sed nisi in augue sodales pulvinar ut et leo. Pellentesque eget leo vitae massa bibendum sollicitudin. Curabitur erat lectus, congue quis auctor sed, aliquet
bibendum est. Ut porta ultricies turpis at maximus. Cras non lobortis justo. Duis rutrum magna sed velit facilisis, et sagittis metus laoreet. Pellentesque quam ligula,
dapibus vitae mauris quis, dapibus cursus leo. Sed sit amet condimentum eros. Nulla vestibulum enim sit amet lorem pharetra, eu fringilla nisl posuere. Sed tristique non
nibh at viverra. Vivamus sed accumsan lacus, nec pretium eros. Mauris elementum arcu eu risus fermentum, tempor ullamcorper neque aliquam. Sed tempor in erat eu
suscipit. In euismod in libero in facilisis. Donec sagittis, odio et fermentum dignissim, risus justo pretium nibh, eget vestibulum lectus metus vel lacus.

Quisque feugiat, magna ac feugiat ullamcorper, augue justo consequat felis, ut fermentum arcu lorem vitae ligula. Quisque iaculis tempor maximus. In quis eros ac tellus
aliquam placerat quis id tellus. Donec non gravida nulla. Morbi faucibus neque sed faucibus aliquam. Sed accumsan mattis nunc, non interdum justo. Cras vitae facilisis
leo. Fusce sollicitudin ultrices sagittis. Maecenas eget massa id lorem dignissim ultrices non et ligula. Pellentesque aliquam mi ac neque tempus ornare. Morbi non enim
vulputate quam ullamcorper finibus id non neque. Quisque malesuada commodo lorem, ut ornare velit iaculis rhoncus. Mauris vel maximus ex.

Morbi eleifend blandit diam, non vulputate ante iaculis in. Donec pellentesque augue id enim suscipit, eget suscipit lacus commodo. Ut vel ex vitae elit imperdiet
vulputate. Nunc eu mattis orci, ut pretium sem. Nam vitae purus mollis ante tempus malesuada a at magna. Integer mattis lectus non luctus lobortis. In a cursus quam,
eget faucibus sem.

Donec vitae condimentum nisi, non efficitur massa. Praesent sed mi in massa sollicitudin iaculis. Pellentesque a libero ultrices, sodales lacus eu, ornare dui. In
laoreet est nec dolor aliquam consectetur. Integer iaculis felis venenatis libero pulvinar, ut pretium odio interdum. Donec in nisi eu dolor varius vestibulum eget vel
nunc. Morbi a venenatis quam, in vehicula justo. Nam risus dui, auctor eu accumsan at, sagittis ac lectus. Mauris iaculis dignissim interdum. Cras cursus dapibus auctor.
Donec sagittis massa vitae tortor viverra vehicula. Mauris fringilla nunc eu lorem ultrices placerat. Maecenas posuere porta quam at semper. Praesent eu bibendum eros.
Nunc congue sollicitudin ante, sollicitudin lacinia magna cursus vitae.
";
