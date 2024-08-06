//! Image


use image::{imageops::FilterType, DynamicImage};

use crate::prelude::*;



// ================================================================================================



// TODO: Different image types (other than halfblocks).
#[derive(Clone)]
pub enum Image {
    Halfblocks {
        data: Vec<(Color, Color)>,
        rect: Rect,
        color_mode: ColorMode,
    },
}

impl Element for Image {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        match self {
            Image::Halfblocks { data, rect, color_mode } => {
                for (i, hb) in data.iter().enumerate() {
                    let x = i as u16 % rect.width;
                    let y = i as u16 / rect.width;
                    if x >= area.width || y >= area.height {
                        continue;
                    }
        
                    let cell = buf.get_mut(area.x + x, area.y + y);
                    let style = cell.style().patch(Style::new()
                        .fg(hb.0)
                        .bg(hb.1)
                        .color_mode(*color_mode));
                    
                    cell.set_style(style)
                        .set_char('▀');
                }
            }
        }
    }
}

impl Image {
    pub fn new(source: &DynamicImage, rect: Rect) -> Self {
        Self::Halfblocks {
            data: encode_halfblocks(source, rect), 
            rect,
            color_mode: ColorMode::overwrite(),
        }
    }

    pub fn color_mode(self, color_mode: ColorMode) -> Self {
        match self {
            Self::Halfblocks { data, rect, .. } => {
                Self::Halfblocks { data, rect, color_mode }
            }
        }
    }

    pub fn get_color_mode(&self) -> ColorMode {
        match self {
            Self::Halfblocks { color_mode, .. } => *color_mode,
        }
    }
}

fn encode_halfblocks(img: &DynamicImage, rect: Rect) -> Vec<(Color, Color)> {
    let img = img.resize_exact(
        rect.width as u32,
        (rect.height * 2) as u32,
        FilterType::Triangle,
    );

    let mut data: Vec<(Color, Color)> = vec![
        (Color::Rgb(0, 0, 0), Color::Rgb(0, 0, 0));
        (rect.width * rect.height) as usize
    ];

    for (y, row) in img.to_rgb8().rows().enumerate() {
        for (x, pixel) in row.enumerate() {
            let position = x + (rect.width as usize) * (y / 2);
            if y % 2 == 0 {
                data[position].0 = Color::Rgb(pixel[0], pixel[1], pixel[2]);
            } else {
                data[position].1 = Color::Rgb(pixel[0], pixel[1], pixel[2]);
            }
        }
    }

    data
}



// ================================================================================================
