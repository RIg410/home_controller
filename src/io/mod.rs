use controller::SwitchHandler;
use controller::DeviceHandler;
use web::WebController;
use std::thread;

mod lora;
mod web;

#[derive(Clone)]
pub struct AppState {
    pub switch: SwitchHandler,
    pub devices: DeviceHandler,
    pub web_controller: WebController,
}

impl AppState {
    pub fn new(switch_handler: SwitchHandler, devices: DeviceHandler, web_controller: WebController) -> AppState {
        AppState { switch: switch_handler.clone(), devices: devices.clone(), web_controller: web_controller.clone() }
    }
}


pub fn start_io(app_state: AppState) {
    let lora_state = app_state.clone();
    let join = thread::spawn(move|| {
        lora::run_lora_ws(lora_state);
    });
    web::run_web_service(app_state);
}