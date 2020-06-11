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
	watch_read: bool,
	registered_in_poll: bool,
	pub token: Token,
	
}

#[derive(Debug)]
pub enum TcpConnectionError {
	Broken,
	Error(String),
	EventError,
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
			watch_read: false,
			registered_in_poll: false,
			token,
		}
	}
	pub fn process_event<F>(&mut self, event: &Event, poll: &mut Poll, on_read_buffer: F) -> Result<(), TcpConnectionError>
		where F: FnMut(&mut NioBuffer) -> Result<(), OnReadBufferError> {
		if event.is_error() {
			return Result::Err(TcpConnectionError::EventError);
		}
		if event.is_readable() {
			self.read(on_read_buffer)?
		}
		if event.is_writable() {
			self.write(poll)?
		}
		Result::Ok(())
	}
	
	pub fn process_read_buffer<F>(&mut self, mut on_read_buffer: F) where F: FnMut(&mut NioBuffer) -> Result<(), OnReadBufferError> {
		self.read_buffer.flip();
		loop {
			self.read_buffer.mark();
			match on_read_buffer(&mut self.read_buffer) {
				Ok(_) => {}
				Err(_) => {
					self.read_buffer.reset().unwrap();
					break;
				}
			}
		}
		self.read_buffer.compact();
	}
	
	
	///
	/// Кодирование команд в буфер для записи
	///
	pub fn prepare_commands_for_send<C, F>(&mut self, poll: &mut Poll, commands: &mut VecDeque<C>, mut command_to_buffer: F) -> Result<(),
		TcpConnectionError>
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
	fn read<F>(&mut self, on_read_buffer: F) -> Result<(), TcpConnectionError>
		where F: FnMut(&mut NioBuffer) -> Result<(), OnReadBufferError> {
		let read_result = self.stream.read(&mut self.read_buffer.to_slice());
		match read_result {
			Ok(0) => {
				Result::Err(TcpConnectionError::Broken)
			}
			Ok(size) => {
				self.read_buffer.set_position(self.read_buffer.position() + size).unwrap();
				self.process_read_buffer(on_read_buffer);
				Result::Ok(())
			}
			Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
				Result::Ok(())
			}
			Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
				Result::Ok(())
			}
			Err(e) => {
				Result::Err(TcpConnectionError::Error(format!("{:?}", e)))
			}
		}
	}
	
	fn write(&mut self, poll: &mut Poll) -> Result<(), TcpConnectionError> {
		if self.write_buffer.has_remaining() {
			let result = self.stream.write(&mut self.write_buffer.to_slice());
			match result {
				Ok(size) => {
					if let Err(e) = self.write_buffer.set_position(self.write_buffer.position() + size) {
						Result::Err(TcpConnectionError::Error(format!("write buffer - error when set new position {:?}", e)))
					} else {
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
					Result::Err(TcpConnectionError::Error(format!("{:?}", e)))
				}
			}
		} else {
			Result::Ok(())
		}
	}
	
	
	///
	/// Подписаться на write события
	///
	pub fn watch_write_and_read(&mut self, poll: &mut Poll) -> Result<(), TcpConnectionError> {
		let interest = Interest::WRITABLE.add(Interest::READABLE);
		match self.watch(poll, interest) {
			Ok(_) => {
				self.watch_read = true;
				Result::Ok(())
			}
			Err(e) => {
				Result::Err(e)
			}
		}
	}
	
	///
	/// Подписаться на read события
	///
	pub fn watch_read(&mut self, poll: &mut Poll) -> Result<(), TcpConnectionError> {
		if !self.watch_read {
			match self.watch(poll, Interest::READABLE) {
				Ok(_) => {
					self.watch_read = true;
					Result::Ok(())
				}
				Err(e) => {
					Result::Err(TcpConnectionError::Error(format!("{:?}", e)))
				}
			}
		} else {
			Result::Ok(())
		}
	}
	
	fn watch(&mut self, poll: &mut Poll, interest: Interest) -> Result<(), TcpConnectionError> {
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
				Result::Err(TcpConnectionError::Error(format!("{:?}", e)))
			}
		}
	}
	
	pub fn stop_watch(&mut self, poll: &mut Poll) -> Result<(), TcpConnectionError> {
		match poll.registry().deregister(&mut self.stream) {
			Ok(_) => {
				Result::Ok(())
			}
			Err(e) => {
				Result::Err(TcpConnectionError::Error(format!("stop watch error {:?}", e)))
			}
		}
	}
}