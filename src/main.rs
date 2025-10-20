use std::mem::MaybeUninit;
use std::time::Duration;

use anyhow::Result;
use drm::buffer::DrmFourcc;
use libcamera::request::ReuseFlag;
use photobooth::camera::{Camera, CameraManager};
use photobooth::display::Display;

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
    state.show_video_stream = true;

    let format_drm = DrmFourcc::Xrgb8888;
    let format_u32 = u32::from_le_bytes([b'X', b'R', b'2', b'4']);

    let mut disp = Display::new("/dev/dri/by-path/platform-gpu-card", format_drm, 24, 32)?;

    let cam_manager = CameraManager::acquire()?;
    let mut camera = Camera::new(&cam_manager, format_u32)?;

    camera.queue_video_requests()?;
    let rx = camera.on_request_receiver();

    loop {
        let mut req: MaybeUninit<libcamera::request::Request> = MaybeUninit::uninit();
        if state.show_video_stream {
            req = MaybeUninit::new(rx.recv_timeout(Duration::from_secs(2))?);

            let fb_ptr = camera.video_stream().get_mapped_buffer(unsafe { req.assume_init_ref() }.cookie());
            unsafe { disp.copy_dma_buf(fb_ptr, camera.video_stream().config().get_frame_size() as usize)? };
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
