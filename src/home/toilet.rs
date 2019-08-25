use devices::{SerialSwitch, Switch as SwitchTrait, SerialDimmer};
use sensors::Switch;
use io::IOBuilder;
use home::Home;
use std::sync::RwLock;
use timer::{Timer, time_ms};
use std::time::Duration;


#[derive(Debug)]
pub struct Toilet {
    pub lamp: SerialDimmer,
    pub fun: SerialSwitch,
    pub switch: Switch,
    pub timer: RwLock<Timer>,
}

impl Toilet {
    pub fn new(io: &mut IOBuilder) -> Toilet {
        Toilet {
            lamp: SerialDimmer::new(io, "toilet_lamp", 0x02, 25, 100),
            fun: SerialSwitch::new(io, "toilet_fun", 0x03),
            switch: Switch::new(io, "toilet", Toilet::on_switch),
            timer: RwLock::new(Timer::new()),
        }
    }

    fn on_switch(home: &Home, is_on: bool) -> Result<(), String> {
        let toilet = &home.toilet;
        if is_on {
            toilet.fun.switch(false);
            toilet.lamp.switch(true);
            toilet.timer.write().unwrap().reset();
        } else {
            if toilet.lamp.is_on() {
                if time_ms() - toilet.switch.last_update() > 30 * 1000 {
                    toilet.fun.switch(true);
                    let fun = toilet.fun.clone();
                    toilet.timer.write().unwrap()
                        .after(Duration::from_secs(60 * 3), move || {
                            fun.switch(false);
                        });
                }
            }

            toilet.lamp.switch(false);
        }
        Ok(())
    }
}