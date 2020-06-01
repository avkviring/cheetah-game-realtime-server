use core::fmt;
use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub enum NioBufferError {
	Overflow,
	NotMarked,
}

macro_rules! write {
		($name:ident, $ty:ty) => {
			#[allow(dead_code)]
			pub fn $name(&mut self, value:$ty)->Result<(),NioBufferError>{
					let size = std::mem::size_of::<$ty>();
					if (self.remaining() < size) {
						Result::Err(NioBufferError::Overflow)
					} else {
						unsafe {
							let src = &value as *const $ty as *const u8;
							let dst = self.buffer[self.position..].as_mut_ptr();
							src.copy_to(dst, size);
						}
					self.position += size;
					Result::Ok(())
				}
			}
	  };
}

macro_rules! read {
		($name:ident, $ty:ty) => {
			#[allow(dead_code)]
			pub fn $name(&mut self)->Result<$ty,NioBufferError>{
					let size = std::mem::size_of::<$ty>();
					if (self.remaining() < size) {
						Result::Err(NioBufferError::Overflow)
					} else {
						let mut result: $ty = Default::default();
						unsafe {
							let src = self.buffer[self.position..].as_ptr();
							let dst = &mut result as *mut $ty as *mut u8;
							src.copy_to(dst, size);
						}
					self.position += size;
					Result::Ok(result)
				}
			}
	  };
}

macro_rules! NioBuffer {
	($name:ident, $size:expr) => {
	
	/// буфер, аналогичный Nio Buffer из Java
	/// на мой взгляд самая удачная реализация сетевого буфера
	pub struct $name {
		buffer: [u8;$size],
		position: usize,
		limit: usize,
		mark: usize,
	}

	impl From<&[u8]> for $name {
		fn from(data: &[u8]) -> Self {
			let mut buffer = $name::new();
			buffer.write_bytes(data);
			buffer
		}
	}

	impl Debug for $name {
		fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
			f
				.debug_struct("$name")
				.field("position", &self.position)
				.field("limit", &self.limit)
				.field("mark", &self.mark)
				.finish()
		}
	}


	impl $name {
		pub const NIO_BUFFER_CAPACITY: usize = $size;
		const NIO_BUFFER_RESET_MARK: usize = $name::NIO_BUFFER_CAPACITY + 1;

	
		pub fn new() -> $name {
			$name {
				buffer: [0; $name::NIO_BUFFER_CAPACITY],
				position: 0,
				limit: $name::NIO_BUFFER_CAPACITY,
				mark: $name::NIO_BUFFER_RESET_MARK,
			}
		}
		
		#[allow(dead_code)]
		pub fn write_bytes(&mut self, data: &[u8]) -> Result<(), NioBufferError> {
			let size = data.len();
			if self.remaining() < size {
				Result::Err(NioBufferError::Overflow)
			} else {
				unsafe {
					let src = data.as_ptr();
					let dst = self.buffer[self.position..].as_mut_ptr();
					src.copy_to(dst, size);
				}
				self.position += size;
				Result::Ok(())
			}
		}
	
		#[allow(dead_code)]
		pub fn read_to_vec_with_u16_size(&mut self) -> Result<Vec<u8>, NioBufferError> {
			let size = self.read_u16()? as usize;
			self.read_to_vec(size)
		}
	
		#[allow(dead_code)]
		pub fn read_to_vec(&mut self, size: usize) -> Result<Vec<u8>, NioBufferError> {
			if self.remaining() < size {
				Result::Err(NioBufferError::Overflow)
			} else {
				let mut result = Vec::with_capacity(size);
				result.extend_from_slice(&self.buffer[self.position..(self.position + size)]);
				self.position += size;
				Result::Ok(result)
			}
		}
	
		/// Переносим данные в начало буфера
		/// после этого буфер готов для записи
		#[allow(dead_code)]
		pub fn compact(&mut self) {
			unsafe {
				let src = self.buffer[self.position..].as_ptr();
				let dst = self.buffer[0..].as_mut_ptr();
				src.copy_to(dst, self.remaining());
			}
			self.position = self.remaining();
			self.limit = $name::NIO_BUFFER_CAPACITY;
		}
	
		#[allow(dead_code)]
		pub fn flip(&mut self) {
			self.limit = self.position;
			self.position = 0;
		}
	
		#[allow(dead_code)]
		pub fn has_remaining(&self) -> bool {
			self.position < self.limit
		}
	
		#[allow(dead_code)]
		pub fn remaining(&self) -> usize {
			self.limit - self.position
		}
	
		#[allow(dead_code)]
		pub fn mark(&mut self) {
			self.mark = self.position;
		}
	
		#[allow(dead_code)]
		pub fn reset(&mut self) -> Result<(), NioBufferError> {
			if self.mark != $name::NIO_BUFFER_RESET_MARK {
				self.position = self.mark;
				self.mark = $name::NIO_BUFFER_RESET_MARK;
				Result::Ok(())
			} else {
				Result::Err(NioBufferError::NotMarked)
			}
		}
		
		#[allow(dead_code)]
		pub fn clear(&mut self) {
			self.position = 0;
			self.limit = $name::NIO_BUFFER_CAPACITY;
		}
		
		#[allow(dead_code)]
		pub fn to_slice(&mut self)->&mut [u8] {
			&mut self.buffer[self.position..self.limit]
		}
		
		#[allow(dead_code)]
		pub fn set_position(&mut self, new_position:usize)->Result<(), NioBufferError> {
			if (new_position > self.limit) {
				Result::Err(NioBufferError::Overflow)
			} else {
				self.position = new_position;
				Result::Ok(())
			}
		}
		
		#[allow(dead_code)]
		pub fn position(&self)->usize {
			self.position
		}
		
		#[allow(dead_code)]
		pub fn set_limit(&mut self, new_limit:usize)->Result<(), NioBufferError> {
			if (self.position > new_limit  || new_limit > $name::NIO_BUFFER_CAPACITY) {
				Result::Err(NioBufferError::Overflow)
			} else {
				self.limit = new_limit;
				Result::Ok(())
			}
		}
	
	
		read!(read_u8,u8);
		read!(read_u16,u16);
		read!(read_u32,u32);
		read!(read_u64,u64);
		read!(read_i8,i8);
		read!(read_i16,i16);
		read!(read_i32,i32);
		read!(read_i64,i64);
		read!(read_f32,f32);
		read!(read_f64,f64);
	
		write!(write_u8, u8);
		write!(write_u16, u16);
		write!(write_u32, u32);
		write!(write_u64, u64);
		write!(write_i8, i8);
		write!(write_i16, i16);
		write!(write_i32, i32);
		write!(write_i64, i64);
		write!(write_f32, f32);
		write!(write_f64, f64);
	}
}
}



NioBuffer!(NioBuffer,1024*100);