use std::time::Duration;

#[derive(Copy, Clone)]
pub struct ProtocolConfiguration {
	// максимальное время ожидания фрейма с другой стороны
	pub disconnect_timeout: Duration,
}
