//! Definition of the StateUpdate type

/// State update messages for the Hyperion service
#[derive(Debug, Clone)]
pub enum StateUpdate {
    ClearAll,
    SolidColor {
        color: palette::LinSrgb,
    },
    Image {
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
}

