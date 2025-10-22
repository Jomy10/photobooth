use std::mem::MaybeUninit;
use std::time::Duration;

use anyhow::Result;
use drm::buffer::DrmFourcc;
use fontdue::layout::{CoordinateSystem, LayoutSettings, TextStyle, Layout};
use libcamera::request::ReuseFlag;
use photobooth::camera::{Camera, CameraManager};
use photobooth::display::Display;
use photobooth::ui::UI;

struct State {
    show_video_stream: bool
}

impl State {
    fn new() -> Self {
        State {
            show_video_stream: false,
        }
    }
}

fn main() -> Result<()> {
    let mut state = State::new();
    // state.show_video_stream = true;

    let format_drm = DrmFourcc::Xrgb8888;
    let format_u32 = u32::from_le_bytes([b'X', b'R', b'2', b'4']);

    let mut disp = Display::new("/dev/dri/by-path/platform-gpu-card", format_drm, 24, 32)?;

    let cam_manager = CameraManager::acquire()?;
    let mut camera = Camera::new(&cam_manager, format_u32)?;

    camera.queue_video_requests()?;
    let rx = camera.on_request_receiver();

    let mut ui = UI::new((disp.size().0 as usize, disp.size().1 as usize));

    let textbox = ui.add_text_box((0., 0.), (100., 100.), fontdue::layout::HorizontalAlign::Left, fontdue::layout::VerticalAlign::Top);
    textbox.borrow_mut().add_text("Hello ", 35.0);
    textbox.borrow_mut().add_text("world!", 40.0);

    loop {
        let mut req: MaybeUninit<libcamera::request::Request> = MaybeUninit::uninit();
        if state.show_video_stream {
            req = MaybeUninit::new(rx.recv_timeout(Duration::from_secs(2))?);

            let fb_ptr = camera.video_stream().get_mapped_buffer(unsafe { req.assume_init_ref() }.cookie());
            unsafe { disp.copy_dma_buf(fb_ptr, camera.video_stream().config().get_frame_size() as usize)? };
        } else {
            disp.clear(0xFF00FF00)?;
        }

        // render UI
        {
            let mut buffer = disp.back_buffer_mut()?;
            let buffer = buffer.as_mut();
            ui.render(buffer);
        }

        // update display
        disp.swap_buffers()?;

        if state.show_video_stream {
            unsafe { req.assume_init_mut() }.reuse(ReuseFlag::REUSE_BUFFERS);
            unsafe { camera.queue_video_request(req.assume_init())? };
        }
    }

    // Ok(())
}

// fn blend_glyph_to_framebuffer(
//     metrics: &fontdue::Metrics,
//     bitmap: &[u8],
//     x: usize,
//     y: usize,
//     fb_size: (usize, usize),
//     fb: &mut [u8]
// ) {
//     for (i, &coverage) in bitmap.iter().enumerate() {
//         if coverage > 0 {
//             let row = i / metrics.width;
//             let col = i % metrics.width;

//             let fb_x = x + col;
//             let fb_y = y + row;

//             if fb_x < fb_size.0 && fb_y < fb_size.1 {
//                 let index = (fb_y * fb_size.0 + fb_x) * 4;

//                 fb[index] = fb[index].saturating_add(f32::round(255. * (coverage as f32) / 255.) as u8);
//                 fb[index + 1] = fb[index + 1].saturating_add(f32::round(255. * (coverage as f32) / 255.) as u8);
//                 fb[index + 2] = fb[index + 2].saturating_add(f32::round(255. * (coverage as f32) / 255.) as u8);
//             }
//         }
//     }
// }
