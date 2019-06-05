//! Debug GUI implementation

use hyperion::hyperion::DebugMessage;
use std::sync::mpsc;

/// Debug GUI module
#[cfg(feature = "gui")]
mod ui {
    use super::*;

    use hyperion::hyperion::StateUpdate;
    use std::thread;
    use std::thread::JoinHandle;

    use std::time::Duration;

    use ::image::{ConvertBuffer, RgbImage};
    use piston_window::*;

    /// Debug GUI window
    pub struct DebugGui {
        /// Thread handle to run the debug GUI
        window_thread: Option<JoinHandle<()>>,
    }

    /// Current GUI mode
    enum GuiMode {
        /// Display a solid color
        SolidColor([f32; 4]),
        /// Display the input image
        Image {
            /// Image as a texture
            texture: G2dTexture,
            /// Image width
            width: u32,
            /// Image height
            height: u32,
        },
    }

    impl Default for GuiMode {
        fn default() -> Self {
            GuiMode::SolidColor([0.05, 0.05, 0.05, 1.0])
        }
    }

    impl DebugGui {
        /// Create a new debug window
        ///
        /// # Parameters
        ///
        /// * `receiver`: receiver for debug messages
        pub fn new(receiver: mpsc::Receiver<DebugMessage>) -> Self {
            let window_thread = Some(thread::spawn(move || {
                let mut window: PistonWindow =
                    WindowSettings::new("hyperiond debug view", [512, 288])
                        .exit_on_esc(true)
                        .automatic_close(true)
                        .build()
                        .unwrap();

                let mut texture_context = window.create_texture_context();

                // Timeout to wait for user events, so we have a chance
                // to mux in the state updates
                let timeout = Duration::from_millis(1);

                let mut state = GuiMode::default();

                while let Some(e) = window.next() {
                    if let Ok(debug_msg) = receiver.recv_timeout(timeout) {
                        // Debug message
                        match debug_msg {
                            DebugMessage::StateUpdate(state_update) => match state_update {
                                StateUpdate::ClearAll => {
                                    state = GuiMode::SolidColor([0., 0., 0., 1.0]);
                                }
                                StateUpdate::SolidColor { color } => {
                                    let (r, g, b) = color.as_rgb();
                                    state = GuiMode::SolidColor([r, g, b, 1.0]);
                                }
                                StateUpdate::Image(raw_image) => {
                                    let (data, width, height) = raw_image.into_raw();

                                    let image_buffer =
                                        RgbImage::from_raw(width, height, data).unwrap();

                                    let texture = G2dTexture::from_image(
                                        &mut texture_context,
                                        &image_buffer.convert(),
                                        &TextureSettings::new().filter(Filter::Nearest),
                                    )
                                    .expect("failed to create texture for incoming image");

                                    state = GuiMode::Image {
                                        texture,
                                        width,
                                        height,
                                    };
                                }
                            },
                            DebugMessage::Terminating => {
                                window.set_should_close(true);
                            }
                        }
                    }

                    let size = window.size();
                    window.draw_2d(&e, |c, g, _| match &state {
                        GuiMode::SolidColor(color) => clear(*color, g),
                        GuiMode::Image {
                            texture,
                            width,
                            height,
                        } => {
                            clear([0.5, 0.5, 0.5, 1.0], g);

                            let scale = size.height / f64::from(*height);
                            let transform = math::multiply(
                                math::multiply(
                                    c.transform,
                                    math::translate([
                                        (size.width - scale * f64::from(*width)) / 2.,
                                        0.,
                                    ]),
                                ),
                                math::scale(scale, scale),
                            );
                            image(texture, transform, g);
                        }
                    });
                }
            }));

            Self { window_thread }
        }
    }

    impl Drop for DebugGui {
        fn drop(&mut self) {
            if let Some(handle) = self.window_thread.take() {
                handle.join().unwrap();
            }
        }
    }

    /// Instantiate a debug listener and associated debug window
    pub fn build_listener() -> (Option<mpsc::Sender<DebugMessage>>, Option<DebugGui>) {
        let (sender, receiver) = mpsc::channel();
        (Some(sender), Some(DebugGui::new(receiver)))
    }
}

/// Debug GUI module
#[cfg(not(feature = "gui"))]
mod ui {
    use super::*;

    /// Dummy debug GUI struct
    pub struct DebugGui;

    /// Instantiate a debug listener and associated debug window
    ///
    /// This method ***will fail*** since this version is built without GUI support.
    pub fn build_listener() -> (Option<mpsc::Sender<DebugMessage>>, Option<DebugGui>) {
        panic!("This version of hyperion.rs does not support the debug GUI. Please recompile with --features=gui");
    }
}

pub use ui::*;
