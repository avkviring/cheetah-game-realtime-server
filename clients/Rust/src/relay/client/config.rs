use crate::relay::client::s2ccommand::UpdateLongCounterS2C;

pub struct ClientConfig {
	update_long_counter_s2c: fn(UpdateLongCounterS2C)
}


impl ClientConfig {
	pub fn new(
		update_long_counter_s2c: fn(UpdateLongCounterS2C)
	) -> ClientConfig {
		ClientConfig {
			update_long_counter_s2c
		}
	}
}