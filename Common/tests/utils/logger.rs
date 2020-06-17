use cheetah_relay_common::utils::logger::LogListener;

fn should_collect_log_error() {
	LogListener::setup_logger();
	log::error!("hello {}", "world");
	let collector = &mut cheetah_relay_common::utils::logger::LOG_COLLECTOR.lock().unwrap();
	let record = collector.items.pop_front().unwrap();
	assert_eq!("(tests/utils/logger.rs in 6) hello world", record.message);
	assert_eq!(log::Level::Error, record.log_level);
}

fn should_collect_log_info() {
	LogListener::setup_logger();
	log::info!("hello {}", "world");
	let collector = &mut cheetah_relay_common::utils::logger::LOG_COLLECTOR.lock().unwrap();
	let record = collector.items.pop_front().unwrap();
	assert_eq!("hello world", record.message);
	assert_eq!(log::Level::Info, record.log_level);
}