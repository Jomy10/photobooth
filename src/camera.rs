use std::os::raw::c_void;
use std::os::fd::RawFd;
use std::pin::Pin;

use libcamera::camera::ActiveCamera;
use anyhow::{anyhow, Result};
use libcamera::camera_manager::CameraList;
use libcamera::framebuffer::AsFrameBuffer;
use libcamera::pixel_format::PixelFormat;
use libcamera::stream::StreamConfigurationRef;
use libcamera::utils::Immutable;
use log::*;
use ouroboros::self_referencing;

#[self_referencing]
pub struct CameraManager {
    camera_manager: libcamera::camera_manager::CameraManager,
    #[borrows(camera_manager)]
    #[covariant]
    cameras: CameraList<'this>,
}

impl CameraManager {
    pub fn acquire() -> Result<Self> {
        Ok(CameraManager::new(
            libcamera::camera_manager::CameraManager::new()?,
            |manager| {
                manager.cameras()
            }
        ))
    }

    pub fn cameras<'a>(&'a self) -> &'a CameraList<'a> {
        self.borrow_cameras()
    }
}

pub struct Camera<'cam> {
    #[allow(unused)]
    camera: Pin<Box<libcamera::camera::Camera<'cam>>>,
    active_camera: libcamera::camera::ActiveCamera<'cam>,
    #[allow(unused)]
    config: Pin<Box<libcamera::camera::CameraConfiguration>>,
    video_stream: VideoStream<'cam>,
}

impl<'cam> Camera<'cam> {
    pub fn new(manager: &'cam crate::camera::CameraManager, format: u32) -> Result<Self> {
        let first_camera = Box::pin(manager.cameras()
            .get(0)
            .ok_or_else(|| anyhow!("No cameras found"))?);

        let camera: *const libcamera::camera::Camera = &* first_camera;

        let mut active_camera = unsafe { &*camera }.acquire()?;

        let mut config = active_camera
            .generate_configuration(&[libcamera::stream::StreamRole::ViewFinder])
            .ok_or_else(|| anyhow!("Couldn't generate configuration"))?;

        let mut stream_cfg = config.get_mut(0).unwrap();
        stream_cfg.set_pixel_format(PixelFormat::new(format, 0));
        // Use 1080p resolution to match display
        stream_cfg.set_size(libcamera::geometry::Size {
            width: 1920,
            height: 1080
        });

        // drop(stream_cfg);

        match config.validate() {
            libcamera::camera::CameraConfigurationStatus::Valid => info!("Camera configuration valid!"),
            libcamera::camera::CameraConfigurationStatus::Adjusted => info!("Camera configuration was adjusted: {config:#?}"),
            libcamera::camera::CameraConfigurationStatus::Invalid => anyhow::bail!("Camera configuration invalid!"),
        }
        // if config.validate().is_valid() {
        //     panic!("Camera configuration validation failed");
        // }
        active_camera.configure(&mut config)?;

        let config = Box::pin(config);

        let stream_cfg = config.get(0).unwrap();
        let stream = stream_cfg.stream().unwrap();
        info!(
            "Video stream: {:?}@{} ({:?})",
            stream_cfg.get_size(),
            stream_cfg.get_stride(),
            stream_cfg.get_pixel_format()
        );

        let config2: *const libcamera::camera::CameraConfiguration = &* config;

        let video_stream = VideoStream::new(stream, unsafe { &*config2 }.get(0).unwrap(), &mut active_camera).unwrap(); // TODO: handle properly

        active_camera.start(None)?;

        Ok(Camera {
            camera: first_camera,
            active_camera,
            config,
            video_stream
        })
    }

    pub fn video_stream(&self) -> &VideoStream<'cam> {
        &self.video_stream
    }

    pub fn video_stream_mut(&mut self) -> &mut VideoStream<'cam> {
        &mut self.video_stream
    }

    #[allow(unused)]
    fn active_camera(&self) -> &ActiveCamera<'cam> {
        &self.active_camera
    }

    #[allow(unused)]
    fn active_camera_mut(&mut self) -> &mut ActiveCamera<'cam> {
        &mut self.active_camera
    }

    pub fn on_request_completed(&mut self, cb: impl FnMut(libcamera::request::Request) + Send + 'cam) {
        self.active_camera_mut().on_request_completed(cb);
    }

    pub fn on_request_receiver(&mut self) -> std::sync::mpsc::Receiver<libcamera::request::Request> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.on_request_completed(move |req| {
            match tx.send(req) {
                Ok(_) => {},
                // TODO: display error
                Err(err) => panic!("Couldn't send request: {:?}", err),
            }
        });
        return rx;
    }

    pub fn queue_video_requests(&mut self) -> Result<()> {
        self.video_stream.queue_requests(&self.active_camera)
    }

    pub fn queue_video_request(&mut self, req: libcamera::request::Request) -> Result<()> {
        self.video_stream.queue_request(&self.active_camera, req)
    }
}

impl<'cam> Drop for Camera<'cam> {
    fn drop(&mut self) {
        self.active_camera.stop().unwrap()
    }
}

pub struct VideoStream<'stream> {
    video_stream: libcamera::stream::Stream,
    // video_buffers: Vec<libcamera::framebuffer_allocator::FrameBuffer>,
    stream_cfg: Immutable<libcamera::stream::StreamConfigurationRef<'stream>>,
    requests: Vec<libcamera::request::Request>,
    mapped_buffers: Vec<*mut c_void>,
}

impl<'stream> VideoStream<'stream> {
    fn new<'a>(
        video_stream: libcamera::stream::Stream,
        stream_cfg: Immutable<libcamera::stream::StreamConfigurationRef<'stream>>,
        cam: &mut libcamera::camera::ActiveCamera<'a>
    ) -> Result<Self> {
        let mut allocator = libcamera::framebuffer_allocator::FrameBufferAllocator::new(cam);
        let buffers = allocator.alloc(&video_stream)?;
        info!("Allocated {} framebuffers", buffers.len());

        let requests: Vec<_> = buffers.into_iter()
            .enumerate()
            .map(|(i, buffer)| {
                let mut request = cam.create_request(Some(i as u64)).expect("Couldn't create request");
                request.add_buffer(&video_stream, buffer)?;
                Ok(request)
            }).collect::<Result<_>>()?;

        let mut camera_buffers_mapped = Vec::with_capacity(requests.len());
        for (i, req) in requests.iter().enumerate() {
            assert!(i == req.cookie() as usize);

            let fb: &libcamera::framebuffer_allocator::FrameBuffer
                = req.buffer(&video_stream).unwrap();
            // NOTE: this code works for formats with only one plane!
            let dma_fd: RawFd = fb.planes().get(0).unwrap().fd();
            let size = stream_cfg.get_frame_size() as usize;

            let fb_ptr = unsafe { libc::mmap(
                std::ptr::null_mut(),
                size,
                libc::PROT_READ,
                libc::MAP_SHARED,
                dma_fd,
                0
            )};

            if fb_ptr == libc::MAP_FAILED {
                anyhow::bail!("Failed to map video buffers to memory");
            }

            camera_buffers_mapped.push(fb_ptr);
        }

        return Ok(Self {
            video_stream,
            // video_buffers: buffers,
            stream_cfg,
            requests,
            mapped_buffers: camera_buffers_mapped,
        });
    }

    pub fn get_mapped_buffer(&self, cookie: u64) -> *mut c_void {
        self.mapped_buffers[cookie as usize]
    }

    pub fn queue_requests(&mut self, camera: &ActiveCamera) -> Result<()> {
        // let cam = camera.active_camera();
        for request in self.requests.drain(..) {
            camera.queue_request(request)?
        }

        Ok(())
    }

    pub fn queue_request(&mut self, camera: &ActiveCamera, request: libcamera::request::Request) -> Result<()> {
        Ok(camera.queue_request(request)?)
    }

    pub fn video_stream(&self) -> &libcamera::stream::Stream {
        &self.video_stream
    }

    pub fn config(&self) -> &Immutable<StreamConfigurationRef<'stream>> {
        &self.stream_cfg
    }

    pub fn max_cookie(&self) -> u64 {
        self.requests.iter().map(|req| req.cookie()).max().unwrap_or(0)
    }

    pub fn requests(&self) -> &Vec<libcamera::request::Request> {
        &self.requests
    }
}

impl<'stream> Drop for VideoStream<'stream> {
    fn drop(&mut self) {
        let size = self.config().get_frame_size() as usize;
        for buffer in self.mapped_buffers.iter() {
            unsafe { libc::munmap(*buffer, size) };
        }
    }
}
