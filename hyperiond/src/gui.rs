#[cfg(feature = "gui")]
mod ui {
    use hyperion::hyperion::{DebugMessage, StateUpdate};
    use std::sync::mpsc;

    use std::thread;
    use std::thread::JoinHandle;

    use std::time::Duration;

    use ::image::{ConvertBuffer, RgbImage};
    use piston_window::*;

    pub struct DebugGui {
        window_thread: Option<JoinHandle<()>>,
    }

    enum GuiMode {
        SolidColor([f32; 4]),
        Image {
            texture: G2dTexture,
            width: u32,
            height: u32,
        },
    }

    impl Default for GuiMode {
        fn default() -> Self {
            GuiMode::SolidColor([0.05, 0.05, 0.05, 1.0])
        }
    }

    impl DebugGui {
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
                                    let (r, g, b) = color.into_components();
                                    state = GuiMode::SolidColor([r, g, b, 1.0]);
                                }
                                StateUpdate::Image {
                                    data,
                                    width,
                                    height,
                                } => {
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

                            let scale = size.height / (*height as f64);
                            let transform = math::multiply(
                                math::multiply(
                                    c.transform,
                                    math::translate([
                                        (size.width - scale * *width as f64) / 2.,
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

    pub fn build_listener() -> (Option<mpsc::Sender<DebugMessage>>, Option<DebugGui>) {
        let (sender, receiver) = mpsc::channel();
        (Some(sender), Some(DebugGui::new(receiver)))
    }
}

#[cfg(not(feature = "gui"))]
mod ui {
    pub fn build_listener() -> ! {
        panic!("This version of hyperion.rs does not support the debug GUI. Please recompile with --features=gui");
    }
}

pub use ui::*;
