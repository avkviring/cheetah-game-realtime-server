use std::collections::VecDeque;
use std::fmt::Debug;
use std::io;
use std::io::{Read, Write};

use mio::{Interest, Poll, Token};
use mio::event::Event;
use mio::net::TcpStream;

use crate::network::niobuffer::{NioBuffer, NioBufferError};

///
/// Чтение/запись/выполнение команд в/из tcp socket
///
#[derive(Debug)]
pub struct TcpConnection {
	stream: TcpStream,
	pub read_buffer: Box<NioBuffer>,
	pub write_buffer: Box<NioBuffer>,
	registered_in_poll: bool,
	pub token: Token,
	
}

#[derive(Debug)]
pub enum ProcessNetworkEventError {
	Broken,
	Error(String),
	EventError,
	Buffer(OnReadBufferError),
}

#[derive(Debug)]
pub enum OnReadBufferError {
	UnknownCommand(u8),
	NioBufferError(NioBufferError),
}


impl TcpConnection {
	pub fn new(stream: TcpStream, buffer_for_read: NioBuffer, token: Token) -> Self {
		let mut buffer_for_write = NioBuffer::new();
		buffer_for_write.flip();
		TcpConnection {
			stream,
			read_buffer: Box::new(buffer_for_read),
			write_buffer: Box::new(buffer_for_write),
			registered_in_poll: false,
			token,
		}
	}
	pub fn process_event<F>(&mut self, event: &Event, poll: &mut Poll, on_read_buffer: F) -> Result<(), ProcessNetworkEventError>
		where F: FnMut(&mut NioBuffer) -> Result<(), OnReadBufferError> {
		log::info!("process_event: {:?}", event);
		if event.is_error() {
			return Result::Err(ProcessNetworkEventError::EventError);
		}
		if event.is_readable() {
			log::info!("process_event: read");
			self.read(on_read_buffer, poll)?
		}
		
		if event.is_writable() {
			log::info!("process_event: write");
			self.write(poll)?
		}
		Result::Ok(())
	}
	
	pub fn process_read_buffer<F: FnMut(&mut NioBuffer) -> Result<(), OnReadBufferError>>(&mut self, mut on_read_buffer: F) -> Result<(), OnReadBufferError> {
		self.read_buffer.flip();
		loop {
			self.read_buffer.mark();
			
			match on_read_buffer(&mut self.read_buffer) {
				Ok(_) => {}
				Err(e) => {
					return match e {
						OnReadBufferError::UnknownCommand(_) => {
							Result::Err(e)
						}
						OnReadBufferError::NioBufferError(_) => {
							self.read_buffer.reset().unwrap();
							self.read_buffer.compact();
							Result::Ok(())
						}
					};
				}
			}
		}
	}
	
	
	///
	/// Кодирование команд в буфер для записи
	///
	pub fn prepare_commands_for_send<C, F>(&mut self, poll: &mut Poll, commands: &mut VecDeque<C>, mut command_to_buffer: F) -> Result<(),
		ProcessNetworkEventError>
		where F: FnMut(&mut NioBuffer, &C) -> Result<(), NioBufferError>, C: Debug {
		if !commands.is_empty() {
			self.write_buffer.compact();
			loop {
				match commands.pop_front() {
					None => { break; }
					Some(command) => {
						self.write_buffer.mark();
						match command_to_buffer(&mut self.write_buffer, &command) {
							Ok(_) => {}
							Err(_) => {
								commands.push_front(command);
								self.write_buffer.reset().unwrap();
								break;
							}
						}
					}
				}
			};
			self.write_buffer.flip();
			self.watch_write_and_read(poll)
		} else {
			Result::Ok(())
		}
	}
	
	///
	/// Читаем, декодируем и исполняем данные из сокета
	///
	fn read<F>(&mut self, on_read_buffer: F, poll: &mut Poll) -> Result<(), ProcessNetworkEventError>
		where F: FnMut(&mut NioBuffer) -> Result<(), OnReadBufferError> {
		let read_result = self.stream.read(&mut self.read_buffer.to_slice());
		let result = match read_result {
			Ok(0) => {
				Result::Err(ProcessNetworkEventError::Broken)
			}
			Ok(size) => {
				self.read_buffer.set_position(self.read_buffer.position() + size).unwrap();
				self.process_read_buffer(on_read_buffer).map_err(ProcessNetworkEventError::Buffer)?;
				Result::Ok(())
			}
			Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
				Result::Ok(())
			}
			Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
				Result::Ok(())
			}
			Err(e) => {
				Result::Err(ProcessNetworkEventError::Error(format!("{:?}", e)))
			}
		};
		self.watch_read(poll);
		result
	}
	
	fn write(&mut self, poll: &mut Poll) -> Result<(), ProcessNetworkEventError> {
		if self.write_buffer.has_remaining() {
			let result = self.stream.write(&mut self.write_buffer.to_slice());
			match result {
				Ok(size) => {
					if let Err(e) = self.write_buffer.set_position(self.write_buffer.position() + size) {
						Result::Err(ProcessNetworkEventError::Error(format!("write buffer - error when set new position {:?}", e)))
					} else {
						log::info!("Connection:write count = {}, remaining = {}", size, self.write_buffer.remaining());
						if !self.write_buffer.has_remaining() {
							self.watch_read(poll)?;
						} else {
							self.watch_write_and_read(poll)?;
						}
						Result::Ok(())
					}
				}
				Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
					Result::Ok(())
				}
				Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
					Result::Ok(())
				}
				Err(e) => {
					Result::Err(ProcessNetworkEventError::Error(format!("{:?}", e)))
				}
			}
		} else {
			Result::Ok(())
		}
	}
	
	
	///
	/// Подписаться на write события
	///
	pub fn watch_write_and_read(&mut self, poll: &mut Poll) -> Result<(), ProcessNetworkEventError> {
		self.watch(poll, Interest::WRITABLE.add(Interest::READABLE))
	}
	
	///
	/// Подписаться на read события
	///
	pub fn watch_read(&mut self, poll: &mut Poll) -> Result<(), ProcessNetworkEventError> {
		self.watch(poll, Interest::WRITABLE.add(Interest::READABLE))
	}
	
	fn watch(&mut self, poll: &mut Poll, interest: Interest) -> Result<(), ProcessNetworkEventError> {
		let result = if self.registered_in_poll {
			poll.registry().reregister(&mut self.stream, self.token.clone(), interest)
		} else {
			poll.registry().register(&mut self.stream, self.token.clone(), interest)
		};
		match result {
			Ok(_) => {
				self.registered_in_poll = true;
				Result::Ok(())
			}
			Err(e) => {
				Result::Err(ProcessNetworkEventError::Error(format!("{:?}", e)))
			}
		}
	}
	
	pub fn stop_watch(&mut self, poll: &mut Poll) -> Result<(), ProcessNetworkEventError> {
		match poll.registry().deregister(&mut self.stream) {
			Ok(_) => {
				Result::Ok(())
			}
			Err(e) => {
				Result::Err(ProcessNetworkEventError::Error(format!("stop watch error {:?}", e)))
			}
		}
	}
}