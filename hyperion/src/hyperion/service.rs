//! Main service future implementation

use futures::{future::FutureExt, select, StreamExt};

use super::{Input, ServiceCommand, ServiceError, StateUpdateKind};
use crate::color::ColorPoint;
use crate::runtime::{Devices, EffectEngine, MuxedInput, PriorityMuxer};
use crate::{config, image, servers};

/// Run the Hyperion service
///
/// This is a future which completes when the service stops.
///
/// # Parameters
///
/// * `json_addr`: address to bind the JSON server to
/// * `proto_addr`: address to bind the Protobuf server to
/// * `config`: parsed service and device configuration
pub async fn run(
    json_addr: impl tokio::net::ToSocketAddrs,
    proto_addr: impl tokio::net::ToSocketAddrs,
    config: config::Config,
) -> Result<(), ServiceError> {
    // Initialize effect engine
    let mut effect_engine = EffectEngine::new(vec!["effects/".into()]);

    // Initialize servers
    let json_rx = servers::bind_json(json_addr, effect_engine.get_definitions()).await?;
    let proto_rx = servers::bind_proto(proto_addr).await?;

    // Channel for effect engine updates
    let (effect_tx, effect_rx) = tokio::sync::mpsc::channel::<Input>(60);

    // Initialize image processor
    let mut image_processor: image::Processor<f32> = Default::default();

    // Initialize devices
    let mut devices = Devices::new(&config);

    // Initialize priority muxer from server and effect inputs
    let mut priority_muxer = PriorityMuxer::new(Box::pin(futures::stream::select_all(vec![
        json_rx, proto_rx, effect_rx,
    ])));

    loop {
        // Process completed future
        select! {
            muxed_input = priority_muxer.next().fuse() => {
                if let Some(input) = &muxed_input {
                    debug!("processing: {}", input);
                }

                match muxed_input {
                    Some(MuxedInput::StateUpdate { update, clear_effects }) => {
                        if clear_effects {
                            effect_engine.clear_all();
                        }

                        let update_time = update.initiated;
                        trace!("muxing latency: {}", update_time.elapsed().as_secs_f64());

                        match update.kind {
                            StateUpdateKind::Clear => {
                                devices.set_all_leds(update_time, ColorPoint::black(), false);
                            },
                            StateUpdateKind::SolidColor { color } => {
                                devices.set_all_leds(update_time, color, false);
                            },
                            StateUpdateKind::Image(raw_image) => {
                                devices.set_from_image(update_time, &mut image_processor, raw_image, false);
                            },
                            StateUpdateKind::LedData(led_data) => {
                                devices.set_leds(update_time, led_data, false);
                            }
                        }

                        trace!("updating latency: {}", update_time.elapsed().as_secs_f64());
                    }
                    Some(MuxedInput::LaunchEffect { effect, deadline }) => {
                        let name = effect.name.clone();
                        let args = effect.args.clone();

                        match effect_engine.launch(
                            effect,
                            deadline,
                            effect_tx.clone(),
                            devices.get_led_count(),
                        ) {
                            Ok(()) => info!(
                                "launched effect {} with args {}",
                                name,
                                args.map(|a| serde_json::to_string(&a).unwrap())
                                    .unwrap_or_else(|| "null".to_owned())
                            ),
                            Err(error) => warn!("failed to launch effect {}: {}", name, error),
                        }
                    }
                    Some(MuxedInput::Internal(service_command)) => {
                        match service_command {
                            ServiceCommand::EffectCompleted { name, result } => {
                                info!("effect '{}' has completed: {:?}", name, result);
                            }
                        }
                    }
                    None => break
                }
            },

            timeout = devices.write_next().fuse() => {}
        };
    }

    Ok(())
}
