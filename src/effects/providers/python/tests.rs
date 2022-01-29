use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use pyo3::{prelude::*, types::PyDict};

use crate::{color::AnsiDisplayExt, effects::EffectDefinition, image::RawImage, models::Color};

use super::{do_run, run, RuntimeMethodError, RuntimeMethods};

fn run_string(
    source: &str,
    args: serde_json::Value,
    methods: impl RuntimeMethods + 'static,
) -> Result<Py<PyDict>, PyErr> {
    do_run(methods, args, |py| {
        let locals = pyo3::types::PyDict::new(py);
        let result = py.run(source, None, Some(locals));
        result.map(|_| locals.into())
    })
}

#[derive(Default)]
struct TestMethodData {
    abort: bool,
    leds: Vec<Color>,
}

#[derive(Default, Clone)]
struct TestMethods(Arc<Mutex<TestMethodData>>);

impl TestMethods {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_led_count(led_count: usize) -> Self {
        Self(Arc::new(Mutex::new(TestMethodData {
            abort: false,
            leds: vec![Color::default(); led_count],
        })))
    }

    pub fn set_abort(&self, abort: bool) {
        self.0.lock().unwrap().abort = abort;
    }
}

impl RuntimeMethods for TestMethods {
    fn get_led_count(&self) -> usize {
        self.0.lock().unwrap().leds.len()
    }

    fn abort(&self) -> bool {
        self.0.lock().unwrap().abort
    }

    fn set_color(&self, color: Color) -> Result<(), RuntimeMethodError> {
        eprintln!("set_color({:?})", color);
        Ok(())
    }

    fn set_led_colors(&self, colors: Vec<crate::models::Color>) -> Result<(), RuntimeMethodError> {
        eprintln!("set_led_colors({})", {
            let mut buf = String::new();
            colors.iter().copied().to_ansi_truecolor(&mut buf);
            buf
        });
        Ok(())
    }

    fn set_image(&self, image: RawImage) -> Result<(), RuntimeMethodError> {
        eprintln!("set_image({:?})", image);
        image.write_to_kitty(&mut std::io::stderr()).unwrap();
        Ok(())
    }
}

#[tokio::test]
async fn test_abort() {
    let tm = TestMethods::new();

    // Start the effect code
    let effect = tokio::task::spawn_blocking({
        let tm = tm.clone();
        move || {
            run_string(
                "import hyperion, time
start = time.time()
while not hyperion.abort():
    pass
duration = time.time() - start
",
                Default::default(),
                tm,
            )
        }
    });

    // Spawn a task that will abort in 500ms
    let seconds = 0.5;
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs_f32(seconds)).await;
        tm.0.lock().unwrap().abort = true;
    });

    // Wait for the effect to complete
    let locals = effect.await.unwrap().unwrap();
    Python::with_gil(|py| {
        assert!(
            (seconds
                - locals
                    .as_ref(py)
                    .get_item("duration")
                    .unwrap()
                    .extract::<f32>()
                    .unwrap())
            .abs()
                < 0.1
        )
    })
}

#[test]
fn test_led_count() {
    let led_count = 12;
    let tm = TestMethods::with_led_count(led_count);

    let result = run_string(
        "import hyperion
leds = hyperion.ledCount
",
        Default::default(),
        tm,
    )
    .expect("failed to run effect code");

    Python::with_gil(|py| {
        assert_eq!(
            led_count,
            result
                .as_ref(py)
                .get_item("leds")
                .unwrap()
                .extract::<usize>()
                .unwrap()
        )
    });
}

async fn run_effect(path: impl AsRef<Path>, duration: Duration) -> Result<(), String> {
    // Resolve effect definition path
    let path = crate::global::Paths::new(None)
        .unwrap()
        .resolve_path(path.as_ref());

    // Read effect definition
    let effect_definition = EffectDefinition::read_file(&path)
        .await
        .expect("failed to read effect definition");

    // Methods
    let tm = TestMethods::with_led_count(12);

    // Spawn timer
    tokio::spawn({
        let tm = tm.clone();
        async move {
            tokio::time::sleep(duration).await;
            tm.set_abort(true);
        }
    });

    // Run effect file with its arguments
    tokio::task::spawn_blocking(move || {
        run(
            effect_definition.script_path().unwrap().as_ref(),
            effect_definition.args.clone(),
            tm,
        )
    })
    .await
    .unwrap()
    .map_err(|err| err.to_string())
}

macro_rules! test_effect {
    ($name:ident, $path:expr) => {
        #[tokio::test]
        async fn $name() {
            assert_eq!(Ok(()), run_effect($path, Duration::from_millis(1000)).await);
        }
    };
}

test_effect!(test_effect_atomic, "$SYSTEM/effects/atomic.json");
test_effect!(test_effect_breath, "$SYSTEM/effects/breath.json");
test_effect!(test_effect_candle, "$SYSTEM/effects/candle.json");
test_effect!(
    test_effect_cinema_fade_in,
    "$SYSTEM/effects/cinema-fade-in.json"
);
test_effect!(
    test_effect_cinema_fade_off,
    "$SYSTEM/effects/cinema-fade-off.json"
);
test_effect!(test_effect_collision, "$SYSTEM/effects/collision.json");
test_effect!(
    test_effect_double_swirl,
    "$SYSTEM/effects/double-swirl.json"
);
test_effect!(test_effect_fire, "$SYSTEM/effects/fire.json");
test_effect!(test_effect_flag, "$SYSTEM/effects/flag.json");
test_effect!(
    test_effect_knight_rider,
    "$SYSTEM/effects/knight-rider.json"
);
test_effect!(test_effect_ledtest, "$SYSTEM/effects/ledtest.json");
test_effect!(test_effect_light_clock, "$SYSTEM/effects/light-clock.json");
test_effect!(test_effect_lights, "$SYSTEM/effects/lights.json");
test_effect!(
    test_effect_mood_blobs_blue,
    "$SYSTEM/effects/mood-blobs-blue.json"
);
test_effect!(
    test_effect_mood_blobs_cold,
    "$SYSTEM/effects/mood-blobs-cold.json"
);
test_effect!(
    test_effect_mood_blobs_full,
    "$SYSTEM/effects/mood-blobs-full.json"
);
test_effect!(
    test_effect_mood_blobs_green,
    "$SYSTEM/effects/mood-blobs-green.json"
);
test_effect!(
    test_effect_mood_blobs_red,
    "$SYSTEM/effects/mood-blobs-red.json"
);
test_effect!(
    test_effect_mood_blobs_warm,
    "$SYSTEM/effects/mood-blobs-warm.json"
);
test_effect!(test_effect_notify_blue, "$SYSTEM/effects/notify-blue.json");
test_effect!(test_effect_pacman, "$SYSTEM/effects/pacman.json");
test_effect!(test_effect_plasma, "$SYSTEM/effects/plasma.json");
test_effect!(
    test_effect_police_lights_single,
    "$SYSTEM/effects/police-lights-single.json"
);
test_effect!(
    test_effect_police_lights_solid,
    "$SYSTEM/effects/police-lights-solid.json"
);
test_effect!(
    test_effect_rainbow_mood,
    "$SYSTEM/effects/rainbow-mood.json"
);
test_effect!(
    test_effect_rainbow_swirl_fast,
    "$SYSTEM/effects/rainbow-swirl-fast.json"
);
test_effect!(
    test_effect_rainbow_swirl,
    "$SYSTEM/effects/rainbow-swirl.json"
);
test_effect!(test_effect_random, "$SYSTEM/effects/random.json");
test_effect!(test_effect_seawaves, "$SYSTEM/effects/Seawaves.json");
test_effect!(test_effect_shutdown, "$SYSTEM/effects/shutdown.json");
test_effect!(test_effect_snake, "$SYSTEM/effects/snake.json");
test_effect!(test_effect_sparks, "$SYSTEM/effects/sparks.json");
test_effect!(test_effect_strobe_red, "$SYSTEM/effects/strobe-red.json");
test_effect!(
    test_effect_strobe_white,
    "$SYSTEM/effects/strobe-white.json"
);
test_effect!(test_effect_traces, "$SYSTEM/effects/traces.json");
test_effect!(
    test_effect_trails_color,
    "$SYSTEM/effects/trails_color.json"
);
test_effect!(test_effect_trails, "$SYSTEM/effects/trails.json");
test_effect!(test_effect_waves, "$SYSTEM/effects/waves.json");
test_effect!(test_effect_x_mas, "$SYSTEM/effects/x-mas.json");
