use cheetah_relay_common::network::command::upload::UploadGameObjectS2CCommand;

fn create_command() -> UploadGameObjectS2CCommand {
	UploadGameObjectS2CCommand {
		cloned_object: GameObject {
			id: 123123,
			owner: Owner::new_root_owner(),
			structures: [(10, DataStruct { data: vec![1, 2, 3, 4, 5] })].iter().cloned().collect(),
			long_counters: [
				(20, LongCounter { counter: 100_500 }),
				(30, LongCounter { counter: 100_501 })].iter().cloned().collect(),
			float_counters: [
				(40, FloatCounter { counter: 100_500.0 }),
				(50, FloatCounter { counter: 100_501.0 }),
				(60, FloatCounter { counter: 100_502.0 })].iter().cloned().collect(),
			groups: AccessGroups::new(),
		}
	}
}