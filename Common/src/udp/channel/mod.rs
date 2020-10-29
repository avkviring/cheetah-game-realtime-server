use std::hash::Hash;
use std::cell::RefCell;
use std::rc::Rc;

///
/// Абстракция для канала данных
/// - две реализации - UDP и Stub (для тестов)
///
pub trait Transport<PeerAddress: Hash> {
	fn create_channel(&self, self_address: PeerAddress) -> Rc<RefCell<dyn Channel<PeerAddress>>>;
}

pub trait Channel<PeerAddress> {
	fn send(&mut self, to: &PeerAddress, buffer: Vec<u8>);
	fn try_recv(&self) -> Option<(PeerAddress, Vec<u8>)>;
}