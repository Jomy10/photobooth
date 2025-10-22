use std::fs::File;
use std::io::{self, BufReader};
use std::mem::MaybeUninit;
use std::time::Duration;

use anyhow::Result;
use drm::buffer::DrmFourcc;
use fontdue::layout::{CoordinateSystem, LayoutSettings, TextStyle, Layout};
use libcamera::request::ReuseFlag;
use log::*;
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

fn configure_logging() -> Result<()> {
    let stdout_log = log4rs::append::console::ConsoleAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("{d(%H:%M:%S)} {h({l})}: {m}\n")))
        .build();

    let logfile = match log4rs::append::rolling_file::RollingFileAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("{d} {M}::{f}:{L} {l}: {m}\n")))
        .build(
            "/var/log/photobooth.log",
            Box::new(log4rs::append::rolling_file::policy::compound::CompoundPolicy::new(
                Box::new(log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger::new(10 * 1024 * 1024)), // 10 MB
                Box::new(log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller::builder().build("/var/log/photobooth.{}.log", 5).unwrap()) // keep 5 old files
            ))
        ) {
            Ok(v) => Some(v),
            Err(err) => match err.kind() {
                io::ErrorKind::PermissionDenied => {
                    eprintln!("Could not access logfile. Create it and correct the permissions.");
                    None
                },
                _ => None
            }
        };

    let mut config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build("stdout", Box::new(stdout_log)));
    if let Some(logfile) = logfile {
        config = config.appender(log4rs::config::Appender::builder().build("logfile", Box::new(logfile)));
    }
    let config = config.build(log4rs::config::Root::builder()
        .appenders(["logfile", "stdout"])
        .build(LevelFilter::Info)
    )?;

    _ = log4rs::init_config(config)?;

    info!("Photobooth logging initialized");

    Ok(())
}

fn get_config() -> Result<photobooth::config::Config> {
    let configuration_path = std::env::var_os("PH_CONFIG").unwrap_or("config.yaml".into());
    info!("Reading configuration from {}", configuration_path.to_string_lossy());
    let config: photobooth::config::Config = if std::fs::exists(&configuration_path)? {
        let config_file = File::open(configuration_path)?;
        let config_reader = BufReader::new(config_file);
        serde_yaml::from_reader(config_reader)?
    } else {
        info!("Configuration file not found, using default");
        Default::default()
    };

    return Ok(config);
}

fn main() -> Result<()> {
    // Set up logging //
    configure_logging()?;

    // Configuration //
    let config = get_config()?;

    // Application code //
    let mut state = State::new();
    // state.show_video_stream = true;

    let format_drm = DrmFourcc::Xrgb8888;
    let format_u32 = u32::from_le_bytes([b'X', b'R', b'2', b'4']);

    info!("Initializing DRM (display)");
    let mut disp = Display::new("/dev/dri/by-path/platform-gpu-card", format_drm, 24, 32)?;

    info!("Initializing camera");
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
