use crate::clients::registry::ClientId;
use crate::ffi::execute_with_client;
use cheetah_common::room::RoomMemberId;

#[no_mangle]
#[allow(unused_must_use)]
pub extern "C" fn set_member_connected_listener(client_id: ClientId, listener: extern "C" fn(RoomMemberId)) -> u8 {
	execute_with_client(client_id, |client| {
		client.listener_member_connected = Some(listener);
		Ok(())
	})
}
