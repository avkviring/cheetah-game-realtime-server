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

/// максимальное количество возможных команд в Frame
/// на самом деле ограничения работают только по размеру, но даже если команда будет один байт -
/// то их не может быть больше 512
pub const CAPACITY_COMMANDS_IN_FRAME: usize = 512;

pub type CommandVec = heapless::Vec<CommandWithChannel, CAPACITY_COMMANDS_IN_FRAME>;
pub const MAX_FRAME_SIZE: usize = 1024;
