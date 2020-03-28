mod serial;
mod web;

use crate::devices::Control;
use crate::home::Home;
pub use crate::io::serial::Cmd;
use crate::io::serial::SerialChannel;
use crate::io::web::WebChannel;
use crate::runtime::Runtime;
use crate::sensors::{ActionType, Switch};
use anyhow::{Error, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Debug, Error as FmtError, Formatter};
use std::sync::Arc;
use std::collections::hash_map::RandomState;

pub trait Input {
    fn update_device(&self, name: &str, value: Value) -> Result<()>;
    fn act(&self, home: &Home, sensor_name: &str, action_type: ActionType) -> Result<()>;
    fn reg_web_devices(&self, ids: Vec<String>, host: String);
    fn devices_list(&self) -> Vec<String>;
    fn get_device(&self, name: &str) -> Result<Value>;
}

pub trait Output {
    fn serial_write(&self, cmd: Cmd) -> Result<()>;
    fn send(&self, id: &str, args: Vec<String>) -> Result<()>;
}

#[derive(Clone)]
pub struct IO {
    serial: SerialChannel,
    web: WebChannel,
    sensors: Option<Arc<SensorsHolder>>,
    devices: Option<Arc<DevicesHolder>>,
    rt: Runtime,
    bg: Option<Arc<Vec<Background>>>,
}

impl IO {
    pub fn create_mut() -> IOBuilder {
        let io = IO {
            serial: SerialChannel::new(),
            web: WebChannel::new(),
            sensors: None,
            devices: None,
            rt: Runtime::new(2),
            bg: None,
        };

        IOMut {
            io,
            sensors: Default::default(),
            devices: Default::default(),
        }
    }

    fn start_bg(&mut self) {
        info!("Start background process");
        let io = self.clone();
        let bg = vec![Background::every(
            &self.rt,
            Duration::from_secs(20),
            true,
            move || io.update_web_devices(),
        )];
        self.bg = Some(Arc::new(bg));
    }

    fn update_web_devices(&self) {
        if let Some(holder) = &self.devices {
            holder
                .devices()
                .iter()
                .for_each(|(_, device)| match device.dev_type() {
                    DeviceType::WebBeam => {
                        log_error!(&device.flush());
                    }
                    _ => {}
                });
        }
    }
}

impl Output for IO {
    fn serial_write(&self, cmd: Cmd) -> Result<()> {
        self.serial.send(cmd)
    }

    fn send(&self, id: &str, args: Vec<String>) -> Result<()> {
        self.web.send(id, args)
    }
}

impl Input for IO {
    fn update_device(&self, name: &str, value: Value) -> Result<()> {
        self.devices.update_device(name, value)
    }

    fn act(&self, home: &Home, sensor_name: &str, action_type: ActionType) -> Result<()> {
        self.sensors.act(home, sensor_name, action_type)
    }

    fn reg_web_devices(&self, ids: Vec<String>, host: String) {
        self.web.reg_device(ids, host);
    }

    fn devices_list(&self) -> Vec<String> {
        self.devices.devices().keys()
            .map(ToOwned::to_owned)
            .collect()
    }

    fn get_device(&self, name: &str) -> Result<Value> {
        self.devices.get_device(name)
    }
}

impl Debug for IO {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "Transport {{}}")
    }
}

pub struct IOMut {
    io: IO,
    sensors: SensorsHolder,
    devices: DevicesHolder,
}

impl IOMut {
    pub fn shared(&self) -> IO {
        self.io.clone()
    }

    pub fn freeze(self) -> IO {
        let IOMut {
            mut io,
            sensors,
            devices,
        } = self;
        let mut io = io;
        io.devices = Some(Arc::new(DevicesHolder { devices }));
        io.sensors = Some(Arc::new(SensorsHolder { sensors }));

        io.start_bg();
        io
    }

    pub fn add_sensor(&mut self, switch: Switch) {
        self.sensors.as_mut().insert(switch.id.as_str().to_owned(), switch);
    }

    pub fn reg_device(&mut self, device: Box<dyn Control>) {
        self.devices.as_mut().insert(device.id().to_owned(), device);
    }

    pub fn rt(&self) -> &Runtime {
        &self.io.rt
    }
}

#[derive(Clone)]
pub struct SensorsHolder {
    sensors: HashMap<String, Switch>,
}

impl SensorsHolder {
    fn act(&self, home: &Home, sensor_name: &str, action_type: ActionType) -> Result<()> {
        if let Some(switch) = self.sensors.get(sensor_name) {
            switch.act(home, action_type)
        } else {
            Err(Error::msg(format!(
                "Sensor with name '{}' not found.",
                sensor_name
            )))
        }
    }
}

impl AsMut<HashMap<String, Switch>> for SensorsHolder {
    fn as_mut(&mut self) -> &mut HashMap<String, Switch, RandomState> {
        &mut self.sensors
    }
}

pub struct DevicesHolder {
    devices: HashMap<String, Box<dyn Control>>,
}

impl DevicesHolder {
    pub fn update_device(&self, name: &str, value: Value) -> Result<()> {
        self.devices
            .get(name)
            .ok_or_else(|| Error::msg(format!("device {} not found", name)))
            .and_then(|dev| dev.update(value))
    }

    pub fn get_device(&self, name: &str) -> Result<Value> {
        self.devices
            .get(name)
            .ok_or_else(|| Error::msg(format!("device {} not found", name)))
            .map(|dev| dev.load())
    }

    pub fn devices(&self) -> &HashMap<String, Box<dyn Control>> {
        &self.devices
    }
}

impl AsMut<HashMap<String, Box<dyn Control>>> for DevicesHolder {
    fn as_mut(&mut self) -> &mut HashMap<String, Box<dyn Control>, RandomState> {
        &mut self.devices
    }
}
