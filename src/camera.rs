use std::fs::File;
use std::io::BufWriter;
use std::os::raw::c_void;
use std::os::fd::RawFd;
use std::pin::Pin;
use std::sync::Arc;

use image::{EncodableLayout, ImageFormat, Rgba};
use libcamera::camera::ActiveCamera;
use anyhow::{anyhow, Result};
use libcamera::camera_manager::CameraList;
use libcamera::framebuffer::AsFrameBuffer;
use libcamera::framebuffer_allocator::FrameBuffer;
use libcamera::pixel_format::PixelFormat;
use libcamera::request::{Request, ReuseFlag};
use libcamera::stream::Stream;
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
    video_stream: VideoStream,
    is_streaming: bool,
    on_request_completed_receiver: std::sync::mpmc::Receiver<libcamera::request::Request>,
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

        match config.validate() {
            libcamera::camera::CameraConfigurationStatus::Valid => info!("Camera configuration valid!"),
            libcamera::camera::CameraConfigurationStatus::Adjusted => {
                warn!("Camera configuration was adjusted: {config:#?}");
                if config.get(0).unwrap().get_pixel_format() != PixelFormat::new(format, 0) {
                    anyhow::bail!("Stream pixel format was changed, it is now incompatible with the screen buffer");
                }
            },
            libcamera::camera::CameraConfigurationStatus::Invalid => anyhow::bail!("Camera configuration invalid!"),
        }

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

        let (tx, rx) = std::sync::mpmc::channel();
        active_camera.on_request_completed(move |req| {
            match tx.send(req) {
                Ok(_) => {},
                // TODO: display error
                Err(err) => panic!("Couldn't send request: {:?}", err),
            }
        });

        Ok(Camera {
            camera: first_camera,
            active_camera,
            config,
            video_stream,
            is_streaming: false,
            on_request_completed_receiver: rx
        })
    }

    pub fn video_stream(&self) -> &VideoStream {
        &self.video_stream
    }

    pub fn video_stream_mut(&mut self) -> &mut VideoStream {
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

    // pub fn on_request_completed(&mut self, cb: impl FnMut(libcamera::request::Request) + Send + 'cam) {
    //     self.active_camera_mut().on_request_completed(cb);
    // }

    pub fn on_request_receiver(&mut self) -> std::sync::mpmc::Receiver<libcamera::request::Request> {
        return self.on_request_completed_receiver.clone();
    }

    fn queue_video_requests(&mut self) -> Result<()> {
        self.video_stream.queue_requests(&self.active_camera)
    }

    fn queue_video_request(&mut self, req: libcamera::request::Request) -> Result<()> {
        self.video_stream.queue_request(&self.active_camera, req)
    }

    fn stop_camera(&mut self) -> Result<()> {
        self.active_camera_mut().stop()?;

        // while let Ok(_request) = self.on_request_completed_receiver.try_recv() {}; // TODO: does any cleanup need to be done here?

        Ok(())
    }

    // TODO: use in init
    /// Stop the camera first if it has been started
    fn configure_video_stream(cam: &mut ActiveCamera, format: PixelFormat) -> Result<VideoStream> {
        trace!("Configuring camera for video stream");

        let mut config = cam
            .generate_configuration(&[libcamera::stream::StreamRole::ViewFinder])
            .ok_or_else(|| anyhow!("Couldn't generate configuration for video stream"))?;

        let mut stream_cfg = config.get_mut(0).unwrap();
        stream_cfg.set_pixel_format(format);
        stream_cfg.set_size(libcamera::geometry::Size {
            // TODO: into parameters in config
            width: 1920,
            height: 1080
        });

        match config.validate() {
            libcamera::camera::CameraConfigurationStatus::Valid => info!("Camera configuration valid!"),
            libcamera::camera::CameraConfigurationStatus::Adjusted => {
                warn!("Camera configuration was adjusted: {config:#?}");
                if config.get(0).unwrap().get_pixel_format() != format {
                    anyhow::bail!("Stream pixel format was changed, it is now incompatible with the screen buffer");
                }
            },
            libcamera::camera::CameraConfigurationStatus::Invalid => anyhow::bail!("Camera configuration invalid!"),
        }

        cam.configure(&mut config)?;

        let stream_cfg = config.get(0).unwrap();
        let stream = stream_cfg.stream().unwrap();
        info!(
            "Video stream: {:?}@{} ({:?})",
            stream_cfg.get_size(),
            stream_cfg.get_stride(),
            stream_cfg.get_pixel_format()
        );

        let config2: *const libcamera::camera::CameraConfiguration = &config;

        let video_stream = VideoStream::new(stream, unsafe { &*config2 }.get(0).unwrap(), cam)?;

        cam.start(None)?;

        trace!("Camera configured for video streaming");

        return Ok(video_stream);
    }

    /// Stop the camera first if it has been started
    fn configure_still_capture(cam: &mut ActiveCamera, format: PixelFormat) -> Result<(FrameBuffer, usize, libcamera::geometry::Size, Stream)> {
        trace!("Configuring camera for still capture");

        let mut config = cam
            .generate_configuration(&[libcamera::stream::StreamRole::StillCapture])
            .ok_or_else(|| anyhow!("Couldn't generate configuration"))?;

        let mut still_cfg = config.get_mut(0).unwrap();
        still_cfg.set_pixel_format(format);
        still_cfg.set_buffer_count(1);

        match config.validate() {
            libcamera::camera::CameraConfigurationStatus::Valid => info!("Camera configuration valid!"),
            libcamera::camera::CameraConfigurationStatus::Adjusted => {
                info!("Camera configuration was adjusted: {config:#?}");
                if config.get(0).unwrap().get_pixel_format() != format {
                    anyhow::bail!("Pixel format changed for still config");
                }
            },
            libcamera::camera::CameraConfigurationStatus::Invalid => anyhow::bail!("Camera configuration invalid!"),
        }

        cam.configure(&mut config)?;

        trace!("Configuration succeeded");

        let still_cfg = config.get(0).unwrap();
        let still_stream = still_cfg.stream().unwrap();

        let mut allocator = libcamera::framebuffer_allocator::FrameBufferAllocator::new(&cam);
        let mut buffers = allocator.alloc(&still_stream)?;
        let buffer = buffers.pop().ok_or_else(|| anyhow!("No buffers allocated for still capture"))?;

        cam.start(None)?;

        trace!("Camera configured for still capture");

        return Ok((buffer, still_cfg.get_frame_size() as usize, still_cfg.get_size(), still_stream));
    }

    pub fn capture(
        &mut self,
        result_file_writer: &mut BufWriter<File>,
        image_format: ImageFormat,
        // Send the mapped fd of the captured image
        on_capture_sender: Option<std::sync::mpsc::Sender<()>>,
        on_image_creation_sender: Option<std::sync::mpsc::Sender<Arc<image::ImageBuffer<Rgba<u8>, Vec<u8>>>>>,
        continue_waiter: Option<std::sync::mpsc::Receiver<()>>,
    ) -> Result<()> {
        trace!("Capturing picture...");

        self.stop_camera()?;

        let cam = self.active_camera_mut();
        // let cam = self.active_camera_mut();

        // cam.stop()?;

        // Configure for StillCapture
        let (buffer, frame_size, img_size, still_stream) = Self::configure_still_capture(cam, PixelFormat::new(u32::from_le_bytes([b'X', b'R', b'2', b'4']), 0))?;

        let mut request = cam.create_request(None).ok_or_else(|| anyhow!("Couldn't create still capture request"))?;
        request.add_buffer(&still_stream, buffer)?;

        trace!("Queueing still capture request {:?}", request);

        cam.queue_request(request)?;

        trace!("Still request submitted");

        let result = self.on_request_completed_receiver.recv()?;
        trace!("Still request result received {:?}", result);
        let buffer: &FrameBuffer = result.buffer(&still_stream).unwrap();
        let fd: RawFd = buffer.planes().get(0).unwrap().fd();
        // let size = still_cfg.get_frame_size() as usize;

        let mapped_fd = unsafe { libc::mmap(
            std::ptr::null_mut(),
            frame_size,
            libc::PROT_READ,
            libc::MAP_SHARED,
            fd,
            0
        ) };

        if mapped_fd == libc::MAP_FAILED {
            anyhow::bail!("Failed to map still buffer to memory");
        }

        let mut img_buffer = image::RgbaImage::new(img_size.width, img_size.height);
        // let img_ptr = img_buffer.as_mut_ptr();

        let img_data = unsafe { std::slice::from_raw_parts(mapped_fd as *const u8, frame_size) };
        assert_eq!(img_buffer.as_bytes().len(), frame_size);

        let img_buffer_slice = unsafe { std::slice::from_raw_parts_mut(img_buffer.as_mut_ptr(), img_buffer.as_bytes().len()) };

        trace!("Copying image data");

        img_buffer_slice.copy_from_slice(img_data);

        if let Some(sender) = on_capture_sender {
            sender.send(())?;
        }

        trace!("Displaying image data and writing to file");

        let img_buffer = Arc::new(img_buffer);
        if let Some(sender) = on_image_creation_sender {
            sender.send(img_buffer.clone())?;
        }

        img_buffer.write_to(result_file_writer, image_format)?;

        trace!("Image written with buffered writer with format {:?}", image_format);

        if let Some(waiter) = continue_waiter {
            waiter.recv()?;
        }

        unsafe { libc::munmap(mapped_fd, frame_size); }

        // Reconfigure for stream
        trace!("Reconfiguring for streaming");
        self.stop_camera()?;
        let format = self.video_stream().get_pixel_format();
        let cam = self.active_camera_mut();

        self.video_stream = Self::configure_video_stream(cam, format)?;

        Ok(())
    }

    pub fn start_stream(&mut self) -> Result<()> {
        if self.is_streaming {
            warn!("Attempted to start streaming while already streaming");
            return Ok(());
        }
        self.is_streaming = true;
        self.queue_video_requests()
        // self.video_stream.queue_requests(&self.active_camera)
    }

    pub fn stop_stream(&mut self) -> Result<()> {
        if !self.is_streaming {
            warn!("Attempted to stop stream while not streaming");
            return Ok(());
        }
        self.is_streaming = false;
        while self.video_stream().requests.len() != self.video_stream().requests_count {
            let req = self.on_request_receiver().recv()?;
            self.video_stream_mut().requests.push(req);
        }
        Ok(())
    }

    /// Requests received from the receiver must be resubmitted
    pub fn resubmit_stream_request(&mut self, mut req: Request) -> Result<()> {
        req.reuse(ReuseFlag::REUSE_BUFFERS);
        if self.is_streaming {
            self.queue_video_request(req)?;
        } else {
            self.video_stream_mut().requests.push(req);
        }

        Ok(())
    }
}

impl<'cam> Drop for Camera<'cam> {
    fn drop(&mut self) {
        self.active_camera.stop().unwrap()
    }
}

pub struct VideoStream {
    video_stream: libcamera::stream::Stream,
    // video_buffers: Vec<libcamera::framebuffer_allocator::FrameBuffer>,
    // stream_cfg: Immutable<libcamera::stream::StreamConfigurationRef<'stream>>,
    frame_size: u32,
    pixel_format: PixelFormat,
    requests: Vec<libcamera::request::Request>,
    requests_count: usize,
    mapped_buffers: Vec<*mut c_void>,
}

impl VideoStream {
    fn new<'a>(
        video_stream: libcamera::stream::Stream,
        stream_cfg: Immutable<libcamera::stream::StreamConfigurationRef<'a>>,
        cam: &mut libcamera::camera::ActiveCamera<'a>
    ) -> Result<Self> {
        let mut allocator = libcamera::framebuffer_allocator::FrameBufferAllocator::new(cam);
        let buffers = allocator.alloc(&video_stream)?;
        debug!("Allocated {} framebuffers for video stream", buffers.len());

        let requests: Vec<_> = buffers.into_iter()
            .enumerate()
            .map(|(i, buffer)| {
                let mut request = cam.create_request(Some(i as u64)).expect("Couldn't create request");
                request.add_buffer(&video_stream, buffer)?;
                Ok(request)
            }).collect::<Result<_>>()?;
        let requests_count = requests.len();

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
            frame_size: stream_cfg.get_frame_size(),
            pixel_format: stream_cfg.get_pixel_format(),
            requests,
            requests_count,
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

    pub fn get_frame_size(&self) -> u32 {
        self.frame_size
    }

    pub fn get_pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }

    // pub fn config(&self) -> &Immutable<StreamConfigurationRef<'stream>> {
    //     &self.stream_cfg
    // }

    pub fn max_cookie(&self) -> u64 {
        self.requests.iter().map(|req| req.cookie()).max().unwrap_or(0)
    }

    pub fn requests(&self) -> &Vec<libcamera::request::Request> {
        &self.requests
    }
}

impl Drop for VideoStream {
    fn drop(&mut self) {
        for buffer in self.mapped_buffers.iter() {
            unsafe { libc::munmap(*buffer, self.frame_size as usize) };
        }
    }
}
