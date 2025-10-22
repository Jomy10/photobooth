use std::rc::Rc;
use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::mpsc::{Receiver, Sender};

use fontdue::layout::{CoordinateSystem, HorizontalAlign, LayoutSettings, VerticalAlign, TextStyle};
use fontdue::Font;
use log::*;

use crate::input::TouchInputEvent;

type Fonts = Rc<[Font; 1]>;
type BoxedUIElement = Box<Rc<RefCell<dyn UIElement>>>;

pub struct UI {
    elements: Vec<BoxedUIElement>,
    size: (usize, usize),
    fonts: Fonts,
    touch_events: Receiver<TouchInputEvent>,
}

impl UI {
    pub fn new(size: (usize, usize)) -> (Self, Sender<TouchInputEvent>) {
        // TODO: replace
        let font = include_bytes!("../SFCamera.ttf") as &[u8];
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

        let (tx, rx) = std::sync::mpsc::channel();

        (Self {
            elements: Vec::new(),
            size,
            fonts: Rc::new([font]),
            touch_events: rx
        }, tx)
    }

    pub fn add_text_box(&mut self, pos: (f32, f32), size: (f32, f32), hor_align: HorizontalAlign, ver_align: VerticalAlign) -> Rc<RefCell<TextBox>> {
        let text_box = Rc::new(RefCell::new(
            TextBox::new(self.fonts.clone(), pos, size, hor_align, ver_align)
        ));

        self.elements.push(Box::new(text_box.clone()));

        return text_box;
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }

    pub fn render(&self, buffer: &mut [u8]) {
        for element in self.elements.iter() {
            element.borrow().render(buffer, self.size)
        }
    }

    pub fn update(&mut self) {
        while let Ok(event) = self.touch_events.try_recv() {
            for element in &self.elements {
                let element = element.borrow();
                if element.is_inside(event.x as f32, event.y as f32) {
                    element.touch_listeners().iter().for_each(|cb| cb());
                }
            }
        }
    }
}

type TouchEventListener = Box<dyn Fn() -> ()>;

pub trait UIElement {
    fn render(&self, buffer: &mut [u8], buffer_size: (usize, usize));
    fn add_touch_listener(&mut self, cb: TouchEventListener);
    fn touch_listeners(&self) -> &Vec<TouchEventListener>;
    fn is_inside(&self, x: f32, y: f32) -> bool;
}

pub struct TextBox {
    layout: fontdue::layout::Layout,
    fonts: Fonts,
    color: u32,
    touch_listeners: Vec<TouchEventListener>,
}

impl TextBox {
    fn new(fonts: Fonts, pos: (f32, f32), size: (f32, f32), hor_align: HorizontalAlign, ver_align: VerticalAlign) -> TextBox {
        trace!("New text box {:?}x{:?}", pos, size);
        let mut layout = fontdue::layout::Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            x: pos.0,
            y: pos.1,
            max_width: Some(size.0),
            max_height: Some(size.0),
            horizontal_align: hor_align,
            vertical_align: ver_align,
            ..Default::default()
        });

        return TextBox {
            layout,
            fonts,
            color: 0xFF0000FF,
            touch_listeners: Vec::new()
        };
    }

    pub fn add_text(&mut self, text: impl AsRef<str>, font_size: f32) {
        self.layout.append(self.fonts.as_slice(), &TextStyle::new(text.as_ref(), font_size, 0));
    }
}

impl UIElement for TextBox {
    fn render(&self, buffer: &mut [u8], buffer_size: (usize, usize)) {
        // let start_x = 0;
        // let start_y = 0;
        for glyph in self.layout.glyphs() {
            let (metrics, bitmap) = self.fonts[0].rasterize(glyph.parent, glyph.key.px);

            let x = glyph.x as usize;
            let y = glyph.y as usize;

            blend_font_grayscale_bitmap_to_buffer(&metrics, &bitmap, self.color, (x, y), buffer_size, buffer);
        }
    }

    fn add_touch_listener(&mut self, cb: TouchEventListener) {
        self.touch_listeners.push(cb)
    }

    fn touch_listeners(&self) -> &Vec<TouchEventListener> {
        &self.touch_listeners
    }

    fn is_inside(&self, x: f32, y: f32) -> bool {
        let set = self.layout.settings();
        return x >= set.x && y >= set.y && x <= set.x + set.max_width.unwrap() && y <= set.y + set.max_height.unwrap();
    }
}

#[inline]
fn blend_font_grayscale_bitmap_to_buffer(
    metrics: &fontdue::Metrics,
    bitmap: &[u8],
    color: u32,
    pos: (usize, usize),
    fb_size: (usize, usize),
    fb: &mut [u8],
) {
    for (i, &coverage) in bitmap.iter().enumerate() {
        if coverage > 0 {
            let row = i / metrics.width;
            let col = i % metrics.width;

            let fb_x = pos.0 + col;
            let fb_y = pos.1 + row;

            if fb_x < fb_size.0 && fb_y < fb_size.1 {
                let index = (fb_y * fb_size.0 + fb_x) * 4;
                fb[index    ] = fb[index    ].saturating_add(f32::round((((color & 0xFF000000)      ) as f32) * ((coverage as f32) / 255.)) as u8);
                fb[index + 1] = fb[index + 1].saturating_add(f32::round((((color & 0x00FF0000) << 8 ) as f32) * ((coverage as f32) / 255.)) as u8);
                fb[index + 2] = fb[index + 2].saturating_add(f32::round((((color & 0x0000FF00) << 16) as f32) * ((coverage as f32) / 255.)) as u8);
            }
        }
    }
}
