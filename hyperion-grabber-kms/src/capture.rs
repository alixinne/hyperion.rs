use std::fs::File;
use std::fs::OpenOptions;
use std::marker::PhantomData;
use std::os::fd::AsRawFd;
use std::os::unix::io::AsFd;
use std::os::unix::io::BorrowedFd;
use std::path::Path;
use std::ptr::null_mut;

use bytes::BytesMut;
use color_eyre::eyre::eyre;
use drm::Device as DrmDevice;
use drm::control::Device as ControlDevice;
use glutin::api::egl::device::Device;
use glutin::api::egl::display::Display;
use glutin::config::{ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::display::AsRawDisplay;
use glutin::prelude::*;
use tracing::debug;

mod graphics;
use graphics::{egl, gl};

use crate::capture::graphics::gl::types::GLuint;

#[derive(Debug)]
/// A simple wrapper for a device node.
struct Card(File);

/// Implementing [`AsFd`] is a prerequisite to implementing the traits found
/// in this crate. Here, we are just calling [`File::as_fd()`] on the inner
/// [`File`].
impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

/// With [`AsFd`] implemented, we can now implement [`drm::Device`].
impl DrmDevice for Card {}
impl ControlDevice for Card {}

impl Card {
    /// Simple helper method for opening a [`Card`].
    fn open(p: impl AsRef<Path>) -> std::io::Result<Self> {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(true);

        // The normal location of the primary device node on Linux
        Ok(Card(options.open(p.as_ref())?))
    }
}

pub struct KmsCapture {
    renderer: graphics::Renderer,
    framebuffer: GLuint,
    renderbuffer: GLuint,
    width: u32,
    height: u32,
    buffer: BytesMut,

    // GL state is bound to the current thread, it can't be sent across threads
    _unsend: PhantomData<*const ()>,
}

impl Drop for KmsCapture {
    fn drop(&mut self) {
        unsafe {
            // Unbind the framebuffer and renderbuffer before deleting.
            self.renderer.BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
            self.renderer.BindRenderbuffer(gl::RENDERBUFFER, 0);
            self.renderer.DeleteFramebuffers(1, &self.framebuffer);
            self.renderer.DeleteRenderbuffers(1, &self.renderbuffer);
        }
    }
}

impl KmsCapture {
    pub fn next_frame(
        &mut self,
        tone_mapping_offset: f32,
        tone_mapping_scaling: f32,
    ) -> (BytesMut, (u32, u32)) {
        self.renderer
            .draw(tone_mapping_offset, tone_mapping_scaling);

        unsafe {
            // Wait for the previous commands to finish before reading from the framebuffer.
            self.renderer.Finish();

            // Download the framebuffer contents to the buffer.
            self.renderer.ReadPixels(
                0,
                0,
                self.width as _,
                self.height as _,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                self.buffer.as_mut_ptr() as *mut _,
            );

            self.buffer.set_len((self.width * self.height * 4) as _);
        }

        (self.buffer.clone(), (self.width, self.height))
    }
}

pub fn init(card_path: &Path, image_width: u32) -> color_eyre::eyre::Result<KmsCapture> {
    // Open device node
    let card = Card::open(card_path)?;

    // Set capabilities
    card.set_client_capability(drm::ClientCapability::Atomic, true)?;
    card.set_client_capability(drm::ClientCapability::UniversalPlanes, true)?;

    let devices = Device::query_devices()?.collect::<Vec<_>>();
    let device = devices
        .iter()
        .find(|d| {
            debug!("{:?}", d);
            if let Some(path) = d.drm_device_node_path() {
                return path == card_path;
            }

            false
        })
        .ok_or(eyre!("device not found"))?;

    // Create a display using the device.
    let display = unsafe { Display::with_device(device, None) }.expect("Failed to create display");

    let template = config_template();
    let config = unsafe { display.find_configs(template) }
        .unwrap()
        .reduce(|config, acc| {
            if config.num_samples() > acc.num_samples() {
                config
            } else {
                acc
            }
        })
        .expect("No available configs");

    debug!("Picked a config with {} samples", config.num_samples());

    // Context creation.
    //
    // In particular, since we are doing offscreen rendering we have no raw window
    // handle to provide.
    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .with_debug(true)
        .build(None);

    let not_current = unsafe {
        display
            .create_context(&config, &context_attributes)
            .unwrap()
    };

    // Make the context current for rendering
    let _context = not_current.make_current_surfaceless().unwrap();

    let renderer = graphics::Renderer::new(&display);
    let (_handle, _info, fb) = card
        .plane_handles()?
        .into_iter()
        .find_map(|handle| {
            let info = card.get_plane(handle).ok()?;
            if !(info.crtc().is_some() && info.framebuffer().is_some()) {
                return None;
            }

            let fb = card
                .get_planar_framebuffer(info.framebuffer().unwrap())
                .ok()?;
            if !(fb.size().0 >= 640 && fb.size().1 >= 480) {
                return None;
            }

            debug!("Plane: {handle:?}");
            debug!("\tCRTC: {:?}", info.crtc());
            debug!("\tFramebuffer: {:?}", info.framebuffer());
            debug!("\tFormats: {:?}", info.formats());

            debug!("Framebuffer: {handle:?}");
            debug!("\tSize: {:?}", fb.size());
            debug!("\tFlags: {:?}", fb.flags());
            debug!("\tPixel format: {:?}", fb.pixel_format());
            debug!("\tModifiers: {:?}", fb.modifier());

            debug!("\tBuffers: {:?}", fb.buffers());
            debug!("\tPitches: {:?}", fb.pitches());
            debug!("\tOffsets: {:?}", fb.offsets());

            Some((handle, info, fb))
        })
        .unwrap();

    let bpo = fb
        .buffers()
        .iter()
        .zip(fb.pitches())
        .zip(fb.offsets())
        .filter_map(|((f, p), o)| f.map(|x| (card.buffer_to_prime_fd(x, 0).unwrap(), p, o)))
        .collect::<Vec<_>>();

    let width = image_width;
    let height = (image_width * fb.size().1) / fb.size().0;

    renderer.resize(width as _, height as _);

    // Create a framebuffer for offscreen rendering since we do not have a window.
    let mut framebuffer = 0;
    let mut renderbuffer = 0;
    unsafe {
        renderer.GenFramebuffers(1, &mut framebuffer);
        renderer.GenRenderbuffers(1, &mut renderbuffer);
        renderer.BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        renderer.BindRenderbuffer(gl::RENDERBUFFER, renderbuffer);
        renderer.RenderbufferStorage(gl::RENDERBUFFER, gl::RGBA8, width as _, height as _);
        renderer.FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::RENDERBUFFER,
            renderbuffer,
        );
    }

    unsafe {
        let display_ptr = match display.raw_display() {
            glutin::display::RawDisplay::Egl(ptr) => ptr,
        };

        let modifier: u64 = fb.modifier().unwrap().into();

        let attribs = if bpo.len() > 1 {
            [
                egl::WIDTH as isize,
                fb.size().0 as _,
                egl::HEIGHT as _,
                fb.size().1 as _,
                egl::LINUX_DRM_FOURCC_EXT as isize,
                fb.pixel_format() as _,
                // Plane 0
                egl::DMA_BUF_PLANE0_FD_EXT as isize,
                bpo[0].0.as_raw_fd() as isize,
                egl::DMA_BUF_PLANE0_PITCH_EXT as isize,
                bpo[0].1 as _,
                egl::DMA_BUF_PLANE0_OFFSET_EXT as isize,
                bpo[0].2 as _,
                egl::DMA_BUF_PLANE0_MODIFIER_LO_EXT as isize,
                (modifier & 0xFFFF_FFFF) as isize,
                egl::DMA_BUF_PLANE0_MODIFIER_HI_EXT as isize,
                (modifier >> 32) as isize,
                // Plane 1
                egl::DMA_BUF_PLANE1_FD_EXT as isize,
                bpo[1].0.as_raw_fd() as isize,
                egl::DMA_BUF_PLANE1_PITCH_EXT as isize,
                bpo[1].1 as _,
                egl::DMA_BUF_PLANE1_OFFSET_EXT as isize,
                bpo[1].2 as _,
                egl::DMA_BUF_PLANE1_MODIFIER_LO_EXT as isize,
                (modifier & 0xFFFF_FFFF) as isize,
                egl::DMA_BUF_PLANE1_MODIFIER_HI_EXT as isize,
                (modifier >> 32) as isize,
                // Plane 2
                egl::DMA_BUF_PLANE2_FD_EXT as isize,
                bpo[2].0.as_raw_fd() as isize,
                egl::DMA_BUF_PLANE2_PITCH_EXT as isize,
                bpo[2].1 as _,
                egl::DMA_BUF_PLANE2_OFFSET_EXT as isize,
                bpo[2].2 as _,
                egl::DMA_BUF_PLANE2_MODIFIER_LO_EXT as isize,
                (modifier & 0xFFFF_FFFF) as isize,
                egl::DMA_BUF_PLANE2_MODIFIER_HI_EXT as isize,
                (modifier >> 32) as isize,
                egl::NONE as isize,
            ]
            .to_vec()
        } else {
            [
                egl::WIDTH as isize,
                fb.size().0 as _,
                egl::HEIGHT as _,
                fb.size().1 as _,
                egl::LINUX_DRM_FOURCC_EXT as isize,
                fb.pixel_format() as _,
                // Plane 0
                egl::DMA_BUF_PLANE0_FD_EXT as isize,
                bpo[0].0.as_raw_fd() as isize,
                egl::DMA_BUF_PLANE0_PITCH_EXT as isize,
                bpo[0].1 as _,
                egl::DMA_BUF_PLANE0_OFFSET_EXT as isize,
                bpo[0].2 as _,
                egl::DMA_BUF_PLANE0_MODIFIER_LO_EXT as isize,
                (modifier & 0xFFFF_FFFF) as isize,
                egl::DMA_BUF_PLANE0_MODIFIER_HI_EXT as isize,
                (modifier >> 32) as isize,
                egl::NONE as isize,
            ]
            .to_vec()
        };

        let image = renderer.egl().CreateImage(
            display_ptr,
            null_mut(),
            egl::LINUX_DMA_BUF_EXT,
            null_mut(),
            attribs.as_ptr(),
        );

        renderer.prepare_texture(image);

        renderer.egl().DestroyImage(display_ptr, image);
    }

    Ok(KmsCapture {
        renderer,
        framebuffer,
        renderbuffer,
        width,
        height,
        buffer: BytesMut::with_capacity((width * height * 4) as _),
        _unsend: Default::default(),
    })
}

fn config_template() -> ConfigTemplate {
    ConfigTemplateBuilder::default()
        .with_alpha_size(8)
        // Offscreen rendering has no support window surface support.
        .with_surface_type(ConfigSurfaceTypes::empty())
        .build()
}
