use std::hash::Hash;

///
/// Абстракция для канала данных
/// - две реализации - UDP и Stub (для тестов)
///
pub trait Transport<PeerAddress: Hash> {
	fn create_channel(&self, self_address: PeerAddress) -> Box<dyn Channel<PeerAddress>>;
}

pub trait Channel<PeerAddress> {
	fn send(&self, to: &PeerAddress, buffer: Vec<u8>);
	fn try_recv(&self) -> Option<(PeerAddress, Vec<u8>)>;
}