use cheetah_relay_common::utils::logger::LogListener;

//#[test]
fn should_collect_log_error() {
	LogListener::setup_logger();
	log::error!("hello {}", "world");
	let collector = &mut cheetah_relay_common::utils::logger::LOG_COLLECTOR.lock().unwrap();
	assert_eq!("[ERROR] (tests/utils/logger.rs in 6) hello world", collector.items.pop_front().unwrap());
}

//#[test]
fn should_collect_log_info() {
	LogListener::setup_logger();
	log::info!("hello {}", "world");
	let collector = &mut cheetah_relay_common::utils::logger::LOG_COLLECTOR.lock().unwrap();
	assert_eq!("[INFO] hello world", collector.items.pop_front().unwrap());
}