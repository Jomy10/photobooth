use std::fs::{File, OpenOptions};
use std::os::fd::AsFd;
use std::os::raw::c_void;

use anyhow::{anyhow, Result};
use drm::buffer::DrmFourcc;
use drm::control::dumbbuffer::{DumbBuffer, DumbMapping};
use drm::control::{framebuffer, Device, FbCmd2Flags};

/// GPU DRM
pub struct Card(File);

impl drm::Device for Card {}
impl drm::control::Device for Card {}
impl AsFd for Card {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.0.as_fd()
    }
}

pub struct Display {
    card: Card,
    connector: drm::control::connector::Handle,
    crtc: drm::control::crtc::Handle,
    mode: drm::control::Mode,
    original_crtc_state: drm::control::crtc::Info,
    size: (u16, u16),

    buffers: [DumbBuffer; 2],
    framebuffers: [framebuffer::Handle; 2],
    current_buffer: usize,
}

impl Display {
    pub fn new(gpu_card: &str, format: DrmFourcc, depth: u32, bpp: u32) -> Result<Self> {
        // Retrieve DRM resources
        let gpu_card = OpenOptions::new()
            .read(true).write(true)
            .open(gpu_card)?;
        let drm = Card(gpu_card);
        let (connector, mode, crtc) = Self::find_drm_resources(&drm)?;
        let original_crtc_state = drm.get_crtc(crtc)?;
        let size = mode.size();

        // Create (frame)buffers
        let buffer1 = drm.create_dumb_buffer((size.0 as u32, size.1 as u32), format, bpp)?;
        let buffer2 = drm.create_dumb_buffer((size.0 as u32, size.1 as u32), format, bpp)?;
        let fb1 = drm.add_framebuffer(&buffer1, depth, bpp)?;
        let fb2 = drm.add_framebuffer(&buffer2, depth, bpp)?;

        return Ok(Self {
            card: drm,
            connector,
            mode,
            crtc,
            original_crtc_state,
            size,
            buffers: [buffer1, buffer2],
            framebuffers: [fb1, fb2],
            current_buffer: 0
        })
    }

    fn find_drm_resources(drm: &impl drm::control::Device) -> Result<(
        drm::control::connector::Handle,
        drm::control::Mode,
        drm::control::crtc::Handle,
    )> {
        let res_handles = drm.resource_handles()?;

        let connector_info = res_handles.connectors()
            .iter()
            .find_map(|&conn| {
                let info = drm.get_connector(conn, false).ok()?; // ignore error and continue
                if info.state() == drm::control::connector::State::Connected {
                    Some(info)
                } else {
                    None
                }
            }).ok_or_else(|| anyhow!("No connected display found"))?;

        let mode = *connector_info.modes().get(0).ok_or_else(|| anyhow!("No modes found for connector"))?;

        // Use the currently active CRTC
        let encoder_handle = connector_info.current_encoder()
            .ok_or_else(|| anyhow!("No active encoder for connector. Is a display connected and active?"))?;
        let encoder_info = drm.get_encoder(encoder_handle)?;

        let crtc_handle = encoder_info.crtc()
            .ok_or_else(|| anyhow!("No active CRTC for encoder."))?;

        println!("Using active CRTC: {:?}", crtc_handle);

        Ok((connector_info.handle(), mode, crtc_handle))
    }

    pub fn display_buffer(&self, buffer: Option<framebuffer::Handle>) -> Result<()> {
        Ok(self.card.set_crtc(self.crtc, buffer, (0, 0), &[self.connector], Some(self.mode))?)
    }

    pub fn display_buffer_at(&self, buffer: Option<framebuffer::Handle>, pos: (u32, u32)) -> Result<()> {
        Ok(self.card.set_crtc(self.crtc, buffer, pos, &[self.connector], Some(self.mode))?)
    }

    pub fn create_dumb_buffer(&self, format: DrmFourcc, bpp: u32) -> Result<DumbBuffer> {
        Ok(self.card.create_dumb_buffer((self.size.0.into(), self.size.1.into()), format, bpp)?)
    }

    pub fn add_framebuffer<B: drm::buffer::Buffer>(&self, buffer: &B, depth: u32, bpp: u32) -> Result<framebuffer::Handle> {
        Ok(self.card.add_framebuffer(buffer, depth, bpp)?)
    }

    pub fn add_planar_framebuffer<B: drm::buffer::PlanarBuffer>(&self, planar_buffer: &B) -> Result<framebuffer::Handle> {
        Ok(self.card.add_planar_framebuffer(planar_buffer, FbCmd2Flags::empty())?)
    }

    pub fn destroy_dumb_buffer(&self, buffer: DumbBuffer) -> Result<()> {
        Ok(self.card.destroy_dumb_buffer(buffer)?)
    }

    pub fn destroy_framebuffer(&self, buffer: framebuffer::Handle) -> Result<()> {
        Ok(self.card.destroy_framebuffer(buffer)?)
    }

    pub fn set_crtc(&self, buffer: Option<framebuffer::Handle>) -> Result<()> {
        match self.card.set_crtc(self.crtc, buffer, (0, 0), &[self.connector], Some(self.mode)) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Failed to set CRTC {:?}: {:?}", self.crtc, e);

                // If this fails due to resource conflict, try to clear the CRTC first
                if e.raw_os_error() == Some(28) {
                    println!("Attempting to clear CRTC first...");
                    // Try to disable the CRTC first, then set it again
                    if let Err(clear_err) = self.card.set_crtc(self.crtc, None, (0, 0), &[], None) {
                        println!("Failed to clear CRTC: {:?}", clear_err);
                    } else {
                        println!("CRTC cleared, retrying...");
                        return Ok(self.card.set_crtc(self.crtc, buffer, (0, 0), &[self.connector], Some(self.mode))?);
                    }
                }

                Err(e.into())
            }
        }
    }

    pub fn device(&self) -> &Card {
        &self.card
    }

    // fn front_buffer(&self) -> DumbBuffer {
    //     self.buffers[self.current_buffer]
    // }

    fn front_framebuffer(&self) -> framebuffer::Handle {
        self.framebuffers[self.current_buffer]
    }

    pub fn back_buffer_mut<'a>(&'a mut self) -> Result<DumbMapping<'a>> {
        let buffer = &mut self.buffers[if self.current_buffer == 0 { 1 } else { 0 }];
        let map = self.card.map_dumb_buffer(buffer)?;
        Ok(map)
    }

    /// Copy dma framebuffer from file descriptor to the back buffer
    /// SAFETY: `fb_ptr` must be valid and of length `size`
    pub unsafe fn copy_dma_buf(&mut self, fb_ptr: *mut c_void, size: usize) -> Result<()> {
        // let mut buffer_map = self.card.map_dumb_buffer(self.back_buffer_mut())?;
        let mut buffer_map = self.back_buffer_mut()?;

        let data = unsafe { std::slice::from_raw_parts(fb_ptr as *const u8, size) };

        let copy_size = size.min(buffer_map.len());
        buffer_map[..copy_size].copy_from_slice(&data[..copy_size]);

        Ok(())
    }

    pub fn swap_buffers(&mut self) -> Result<()> {
        self.current_buffer = if self.current_buffer == 1 { 0 } else { 1 };
        self.set_crtc(Some(self.front_framebuffer()))
    }

    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    fn buffer_size_u8(&self) -> usize {
        (self.size.0 as usize) * (self.size.1 as usize) * 4
    }

    fn buffer_size_u32(&self) -> usize {
        (self.size.0 as usize) * (self.size.1 as usize)
    }

    pub fn clear(&mut self, color: u32) -> Result<()> {
        let buffer_size = self.buffer_size_u32();
        let mut back_buffer = self.back_buffer_mut()?;
        let back_buffer: &mut [u32] = unsafe { std::slice::from_raw_parts_mut(back_buffer.as_mut_ptr() as *mut u32, buffer_size) };
        back_buffer[..].fill(color);
        Ok(())
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        for buffer in self.buffers.into_iter() {
            self.card.destroy_dumb_buffer(buffer).unwrap();
        }

        for framebuffer in self.framebuffers.into_iter() {
            self.card.destroy_framebuffer(framebuffer).unwrap();
        }

        self.card.set_crtc(
            self.original_crtc_state.handle(),
            self.original_crtc_state.framebuffer(),
            self.original_crtc_state.position(),
            &[self.connector],
            self.original_crtc_state.mode()
        ).unwrap();
    }
}
