use std::collections::BTreeMap;

use tokio::sync::broadcast::Receiver;

use crate::{
    global::{Global, InputMessage, InputMessageData, InputSourceHandle, Message, MuxedMessage},
    models::Color,
};

pub struct PriorityMuxer {
    receiver: Receiver<InputMessage>,
    source_handle: InputSourceHandle<MuxedMessage>,
    inputs: BTreeMap<i32, InputMessage>,
}

const MAX_PRIORITY: i32 = 256;

impl PriorityMuxer {
    pub async fn new(global: Global) -> Self {
        let mut this = Self {
            receiver: global.subscribe_input().await,
            source_handle: global
                .register_muxed_source("PriorityMuxer".to_owned())
                .await
                .unwrap(),
            inputs: Default::default(),
        };

        // Start by clearing all outputs
        this.clear_all().await;

        this
    }

    fn current_priority(&self) -> i32 {
        *self.inputs.keys().next().unwrap()
    }

    fn notify_output_change(&mut self) {
        debug!("current output changed, notifying consumers");

        // unwrap: there is always at least one input
        let target = self.inputs.values().next().unwrap();

        match self.source_handle.send(target.data().clone().into()) {
            Ok(_) => {}
            Err(error) => {
                warn!("error forwarding muxed message: {:?}", error);
            }
        }
    }

    async fn clear_all(&mut self) {
        self.inputs.clear();
        debug!("cleared all inputs");

        self.inputs.insert(
            MAX_PRIORITY,
            InputMessage::new(
                self.source_handle.id(),
                InputMessageData::SolidColor {
                    priority: MAX_PRIORITY,
                    duration: None,
                    color: Color::from_components((0, 0, 0)),
                },
            ),
        );

        debug!("current priority is now {}", self.current_priority());
        self.notify_output_change()
    }

    async fn clear(&mut self, priority: i32) {
        assert!(priority < MAX_PRIORITY);
        let notify = self.current_priority() == priority;

        self.inputs.remove(&priority);
        debug!("cleared priority {}", priority);

        if notify {
            debug!("current priority is now {}", self.current_priority());
            self.notify_output_change()
        }
    }

    async fn handle_input(&mut self, input: InputMessage) {
        let priority = input.data().priority().unwrap();
        let is_new = priority < self.current_priority();
        let notify = priority <= self.current_priority();

        let before = self.inputs.insert(priority, input.clone());
        trace!(
            "new command for priority {}: {:?}, replaces: {:?}",
            priority,
            input,
            before
        );

        if is_new {
            debug!("current priority is now {}", priority);
        }

        if notify {
            self.notify_output_change()
        }
    }

    pub async fn run(mut self) {
        loop {
            match self.receiver.recv().await {
                Ok(input) => {
                    trace!("got input: {:?}", input);

                    // Check if this will change the output
                    match input.data() {
                        InputMessageData::ClearAll => self.clear_all().await,
                        InputMessageData::Clear { priority } => self.clear(*priority).await,
                        InputMessageData::SolidColor { .. } => self.handle_input(input).await,
                        InputMessageData::Image { .. } => self.handle_input(input).await,
                    }
                }
                Err(error) => {
                    error!("could not get input: {:?}", error);
                }
            }
        }
    }
}
