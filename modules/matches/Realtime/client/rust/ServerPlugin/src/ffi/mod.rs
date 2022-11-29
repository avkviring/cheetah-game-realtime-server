use std::ffi::CStr;
use std::fmt::Display;
use std::os::raw::c_char;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::registry::{Registry, ServerPlugin, ServerPluginId};

lazy_static! {
	static ref REGISTRY: Mutex<Registry> = Mutex::new(Default::default());
	static ref LAST_ERROR: Mutex<String> = Mutex::new(String::new());
}

#[repr(u8)]
pub enum ResultCode {
    OK = 0,
    Error = 1,
}

#[no_mangle]
pub unsafe extern "C" fn create_plugin(grpc_server_addr: *const c_char, out_plugin_id: &mut ServerPluginId) -> ResultCode {
    let grpc_server_addr_as_string = CStr::from_ptr(grpc_server_addr).to_str().unwrap().to_string();
    execute(|registry| {
        let plugin = ServerPlugin::new(grpc_server_addr_as_string)
        match plugin {
            Ok(plugin) => {
                let id = registry.register_plugin(plugin);
                *out_plugin_id = id;
                Ok(())
            }
            Err(err) => {
                Err(err)
            }
        }
    })
}


#[no_mangle]
pub unsafe extern "C" fn get_new_events()


pub fn execute<F, R, E: Display>(body: F) -> ResultCode
    where
        F: FnOnce(&mut Registry) -> Result<R, E>,
{
    let mut lock = REGISTRY.lock();
    match lock.as_mut() {
        Ok(registry) => match body(registry) {
            Ok(_) => ResultCode::OK,
            Err(e) => {
                set_error_msg(format!("{}", e));
                ResultCode::Error
            }
        },
        Err(e) => {
            set_error_msg(format!("{:?}", e));
            ResultCode::Error
        }
    }
}

fn set_error_msg(msg: String) {
    let mut last_error = LAST_ERROR.lock().unwrap();
    *last_error = msg;
}
