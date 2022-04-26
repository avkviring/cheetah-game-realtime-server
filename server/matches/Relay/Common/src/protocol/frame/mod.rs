use crate::protocol::frame::applications::CommandWithChannel;

pub mod applications;
pub mod channel;
pub mod headers;
pub mod input;
pub mod output;
///
/// Уникальный возрастающий идентификатор фрейма
/// - игнорируем уже принятый фрейм с таким же frame_id
/// - используется как nonce в алгоритме шифрования
/// - должен быть уникальным, даже если это повторно отсылаемый фрейм
///
pub type FrameId = u64;
pub const MAX_COMMAND_IN_FRAME: usize = 20;
pub type CommandVec = heapless::Vec<CommandWithChannel, MAX_COMMAND_IN_FRAME>;
pub const MAX_FRAME_SIZE: usize = 1024;
