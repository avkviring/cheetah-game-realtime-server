#[macro_use]
extern crate lazy_static;

use std::cell::{RefCell, RefMut};
use std::sync::{Arc, Mutex};

use crate::relay::api::API;
use crate::relay::client::config::ClientConfig;
use crate::relay::client::s2ccommand::UpdateLongCounterS2C;

pub mod relay;

#[no_mangle]
fn calc(a: u16, b: u16) -> u16 {
	a + b * 2
}


lazy_static! {
    static ref API_REF: Arc<Mutex<API>> = Arc::new(Mutex::new(API::new()));
}

fn execute<F, T>(body: F) -> T
	where
		F: FnOnce(&mut API) -> T
{
	let api = API_REF.clone();
	let api = api.lock();
	let api = &mut *(api.unwrap());
	body(api)
}

/// TODO могут быть проблемы с многопоточной инициализацией
#[no_mangle]
pub extern "C" fn init_library() {}

///
/// Создать клиента и получить его идентификатор
///
#[no_mangle]
pub extern "C" fn create_client(
	update_long_counter_s2c: fn(UpdateLongCounterS2C)
) -> u16 {
	let config = ClientConfig::new(update_long_counter_s2c);
	execute(|api| api.create_client(config))
}

///
/// Создать клиента и получить его идентификатор
///
#[no_mangle]
pub extern "C" fn collect_s2c_commands(client_id: u16) {
	execute(|api| api.collect_s2c_commands(client_id));
}

#[no_mangle]
pub extern "C" fn destroy_client(client_id: u16) {
	execute(|api| api.destroy_client(client_id));
}

