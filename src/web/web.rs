use web::AppState;
use actix_web::{server, App, http, Path, State, Result as WebResult, Json};
use chrono::Local;
use io::Input;
use sensors::ActionType;
use serde_json::Value;
use home::Runner;

pub fn run_web_service(state: AppState) {
    server::new(move || {
        App::with_state(state.clone())
            .prefix("/odin/api")
            .resource("switch/{switch}/{state}", |r| r.method(http::Method::GET).with(switch_hndl))
            .resource("update/{device}", |r| r.method(http::Method::POST).with(update_device))
            .resource("devices/list", |r| r.method(http::Method::POST).with(devices_list))
            .resource("device/{device}", |r| r.method(http::Method::GET).with(get_device))
            .resource("reg-device/{ids}/{base_url}", |r| r.method(http::Method::GET).with(reg_device))
            .resource("script/{name}", |r| r.method(http::Method::GET).with(run_script))
            .resource("time", |r| r.method(http::Method::GET).with(get_time))
    })
        .bind("0.0.0.0:1884")
        .expect("Can not bind to port 1884")
        .run();
}

fn switch_hndl((params, state): (Path<(String, String)>, State<AppState>)) -> WebResult<String> {
    if let Err(err) = state.io.act(&state.home, &params.0, ActionType::Toggle) {
        println!("toggle switch:{} err: {}", &params.0, err);
        Ok(err)
    } else {
        println!("toggle switch:{} ok", &params.0);
        Ok("Ok".to_owned())
    }
}

fn update_device((params, state, value): (Path<(String)>, State<AppState>, Json<Value>)) -> WebResult<String> {
    println!("update device:{}, value: {:?}", &params, &value);
    if let Err(err) = state.update_device(&params, value.0) {
        println!("update device err: {}", err);
        Ok(err)
    } else {
        Ok("Ok".to_owned())
    }
}

fn devices_list(state: State<AppState>) -> WebResult<Json<Vec<String>>> {
    Ok(Json(state.devices_list()))
}

fn get_device((params, state): (Path<(String)>, State<AppState>)) -> WebResult<Json<Value>> {
    match state.get_device(&params) {
        Ok(val) => Ok(Json(val)),
        Err(err) => {
            println!("get device err: {}", err);
            Ok(Json(json!({"err": err})))
        }
    }
}

/// 0 - ids (id_1:id_2:id_3)
/// 1 - base_url (host:port)
fn reg_device((params, state): (Path<(String, String)>, State<AppState>)) -> WebResult<String> {
    println!("reg device id:{:?}, ip: {}", &params.0, &params.1);
    let ids = params.0.split(":")
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();
    let host = params.1.to_owned();

    state.io.reg_web_devices(ids, host);
    Ok("Ok".to_owned())
}

fn get_time(_state: State<AppState>) -> WebResult<String> {
    let time = Local::now();
    Ok(time.to_rfc2822())
}

fn run_script((params, state): (Path<(String)>, State<AppState>)) -> WebResult<String> {
    println!("run script:{:?}", &params);
    state.home.run_script(&params);
    Ok("Ok".to_owned())
}