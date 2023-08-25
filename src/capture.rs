use anyhow::{anyhow, Result};
use image::RgbaImage;
use crate::capture_screen;
use crate::capture_screen_area;

pub use display_info::DisplayInfo;

pub fn vec_to_rgba_image(width: u32, height: u32, buf: Vec<u8>) -> Result<RgbaImage> {
    RgbaImage::from_vec(width, height, buf).ok_or(anyhow!("buffer not big enough"))
}

#[cfg(any(target_os = "windows", target_os = "macos", test))]
pub fn bgra_to_rgba_image(width: u32, height: u32, buf: Vec<u8>) -> Result<RgbaImage> {
    // convert to rgba
    let rgba_buf = buf
        .chunks_exact(4)
        .take((width * height) as usize)
        .flat_map(|bgra| [bgra[2], bgra[1], bgra[0], bgra[3]])
        .collect();

    vec_to_rgba_image(width, height, rgba_buf)
}

#[derive(Debug, Clone, Copy)]
pub struct Screen {
    pub display_info: DisplayInfo,
}

impl Screen {
    pub fn new(display_info: &DisplayInfo) -> Self {
        Screen {
            display_info: *display_info,
        }
    }

    pub fn all() -> Result<Vec<Screen>> {
        let screens = DisplayInfo::all()?.iter().map(Screen::new).collect();
        Ok(screens)
    }

    pub fn from_point(x: i32, y: i32) -> Result<Screen> {
        let display_info = DisplayInfo::from_point(x, y)?;
        Ok(Screen::new(&display_info))
    }

    pub fn capture(&self) -> Result<RgbaImage> {
        capture_screen(&self.display_info)
    }

    /**
     * 截取指定区域
     * 区域x,y为相对于当前屏幕的x,y坐标
     */
    pub fn capture_area(&self, x: i32, y: i32, width: u32, height: u32) -> Result<RgbaImage> {
        let display_info = self.display_info;
        let screen_x2 = display_info.x + display_info.width as i32;
        let screen_y2 = display_info.y + display_info.height as i32;

        let mut x1 = x + display_info.x;
        let mut y1 = y + display_info.y;
        let mut x2 = x1 + width as i32;
        let mut y2 = y1 + height as i32;

        // x y 必须在屏幕范围内
        if x1 < display_info.x {
            x1 = display_info.x;
        } else if x1 > screen_x2 {
            x1 = screen_x2
        }

        if y1 < display_info.y {
            y1 = display_info.y;
        } else if y1 > screen_y2 {
            y1 = screen_y2;
        }

        if x2 > screen_x2 {
            x2 = screen_x2;
        }

        if y2 > screen_y2 {
            y2 = screen_y2;
        }

        if x1 >= x2 || y1 >= y2 {
            return Err(anyhow!("Area size is invalid"));
        }

        capture_screen_area(
            &display_info,
            x1 - display_info.x,
            y1 - display_info.y,
            (x2 - x1) as u32,
            (y2 - y1) as u32,
        )
    }
}

/// Some platforms e.g. MacOS can have extra bytes at the end of each row.
///
/// See
/// https://github.com/nashaofu/screenshots-rs/issues/29
/// https://github.com/nashaofu/screenshots-rs/issues/38
#[cfg(any(target_os = "macos", test))]
pub fn remove_extra_data(width: usize, bytes_per_row: usize, buf: Vec<u8>) -> Vec<u8> {
    buf.chunks_exact(bytes_per_row)
        .flat_map(|row| row.split_at(width * 4).0.to_owned())
        .collect()
}

#[cfg(any(target_os = "linux", test))]
pub fn png_to_rgba_image(
    filename: &String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<RgbaImage> {
    use image::open;

    let mut dynamic_image = open(filename)?;
    dynamic_image = dynamic_image.crop(x as u32, y as u32, width as u32, height as u32);
    Ok(dynamic_image.to_rgba8())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bgra() {
        let image = bgra_to_rgba_image(2, 1, vec![1, 2, 3, 4, 255, 254, 253, 252]).unwrap();
        assert_eq!(
            image,
            RgbaImage::from_vec(2, 1, vec![3, 2, 1, 4, 253, 254, 255, 252]).unwrap()
        );
    }

    #[test]
    fn extra_data() {
        let clean = remove_extra_data(
            2,
            9,
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15, 16, 17, 18, 19,
            ],
        );
        assert_eq!(
            clean,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16, 17, 18]
        );
    }
}
