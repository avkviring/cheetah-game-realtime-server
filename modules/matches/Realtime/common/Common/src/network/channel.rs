use std::io;
use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::time::Instant;

use crate::network::bind_to_free_socket;
use crate::network::emulator::NetworkLatencyEmulator;

#[derive(Debug)]
pub struct NetworkChannel {
	socket: UdpSocket,
	emulator: Option<NetworkLatencyEmulator>,
	pub recv_packet_count: u64,
	pub send_packet_count: u64,
	pub recv_size: u64, // размер всех принятых данных
	pub send_size: u64, //размер всех отправленных данных
}

impl NetworkChannel {
	pub fn new() -> io::Result<Self> {
		let socket = bind_to_free_socket()?;
		socket.set_nonblocking(true)?;
		Ok(Self {
			socket,
			emulator: None,
			recv_packet_count: 0,
			send_packet_count: 0,
			recv_size: 0,
			send_size: 0,
		})
	}

	pub fn recv(&mut self, now: Instant, buf: &mut [u8]) -> io::Result<usize> {
		let result = self.socket.recv(buf);
		if result.is_ok() {
			self.recv_packet_count += 1;
			self.recv_size += *result.as_ref().unwrap() as u64;
		}

		if let Some(emulator) = self.emulator.as_mut() {
			// полученный пакет из сети сохраняем в эмуляторе
			if let Ok(size) = result {
				emulator.schedule_in(now, &buf[0..size]);
			}
			// вместо полученного пакета получаем пакет из очереди с учетом эмуляции сети
			match emulator.get_in(now) {
				None => Err(Error::new(ErrorKind::WouldBlock, "")),
				Some(buffer) => {
					buf[0..buffer.len()].copy_from_slice(buffer.as_slice());
					Ok(buffer.len())
				}
			}
		} else {
			result
		}
	}

	pub fn send_to(&mut self, now: Instant, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
		self.send_packet_count += 1;
		self.send_size += buf.len() as u64;
		match &mut self.emulator {
			None => self.socket.send_to(buf, addr),
			Some(emulator) => {
				emulator.schedule_out(now, buf, addr);
				Ok(buf.len())
			}
		}
	}

	///
	/// Если в эмуляторе есть данные для отправки в реальный сокет - отправляем
	///
	pub fn cycle(&mut self, now: Instant) {
		if let Some(emulator) = self.emulator.as_mut() {
			while let Some((buffer, addr)) = emulator.get_out(now) {
				match self.socket.send_to(buffer.as_slice(), addr) {
					Ok(_) => {}
					Err(e) => {
						tracing::error!("[NetworkChannel] emulate mode, send to socket error {:?}", e);
					}
				}
			}
		}
	}

	///
	/// Сконфигурировать эмулятор
	///
	pub fn config_emulator<T>(&mut self, f: T)
	where
		T: FnOnce(&mut NetworkLatencyEmulator),
	{
		if self.emulator.is_none() {
			self.emulator.replace(NetworkLatencyEmulator::default());
		}

		if let Some(emulator) = self.emulator.as_mut() {
			f(emulator);
		}
	}

	///
	/// Отключить эмуляцию характеристик сети
	/// - все не отправленные и не принятые пакеты будут потеряны
	/// - восстановление данных возлагается на Relay протокол
	///
	pub fn reset_emulator(&mut self) {
		self.emulator = None;
	}
}

#[cfg(test)]
pub mod tests {
	use std::ops::Add;
	use std::time::{Duration, Instant};

	use crate::network::channel::NetworkChannel;

	///
	/// Проверяем работу канала в обычном режиме
	///
	#[test]
	fn should_receive_and_send() {
		let mut channel_a = NetworkChannel::new().unwrap();
		let mut channel_b = NetworkChannel::new().unwrap();

		let now = Instant::now();
		let send_data = vec![1, 2, 3];
		channel_a
			.send_to(now, send_data.as_slice(), channel_b.socket.local_addr().unwrap())
			.unwrap();
		std::thread::sleep(Duration::from_millis(10));
		let mut recv_data = [0; 1024];
		assert!(matches!(channel_b.recv(now, &mut recv_data), Ok(size) if send_data.len()==size));
	}

	///
	/// Проверяем интеграцию канала и системы эмулирования характеристик сети
	/// Проверяем только rtt, этого достаточно, так как мы проверяем только интеграцию
	/// Все остальные тесты есть в [`NetworkLatencyEmulator`]
	///
	#[test]
	fn should_receive_and_send_with_emulator() {
		let mut channel_a = NetworkChannel::new().unwrap();
		let rtt = Duration::from_millis(100);
		let half_rtt = rtt.div_f64(2.0);
		channel_a.config_emulator(|emulator| {
			emulator.configure_rtt(rtt, 0.0);
		});
		let mut channel_b = NetworkChannel::new().unwrap();
		channel_b.config_emulator(|emulator| {
			emulator.configure_rtt(rtt, 0.0);
		});

		let now = Instant::now();
		let send_data = vec![1, 2, 3];
		channel_a
			.send_to(now, send_data.as_slice(), channel_b.socket.local_addr().unwrap())
			.unwrap();

		// данных нет - так как включен эмулятор лага
		std::thread::sleep(Duration::from_millis(10));
		let mut recv_data = [0; 1024];
		assert!(matches!(channel_b.recv(now, &mut recv_data), Err(_)));
		// время окончания эмуляции rtt
		let after_rtt_time = now.add(half_rtt).add(Duration::from_millis(10));
		// данные должны быть отправлены, но их еще не будет на принимающей стороне
		channel_a.cycle(after_rtt_time);
		std::thread::sleep(Duration::from_millis(10));
		assert!(matches!(channel_b.recv(now, &mut recv_data), Err(_)));

		// а теперь будут, так как прошло время эмуляции rtt
		assert!(matches!(channel_b.recv(after_rtt_time, &mut recv_data), Ok(size) if send_data.len()==size));
	}

	///
	/// Проверяем как собирается статистика по количеству и размеру пакетов
	///
	#[test]
	fn should_statistics() {
		let mut channel_a = NetworkChannel::new().unwrap();
		let mut channel_b = NetworkChannel::new().unwrap();

		assert_eq!(channel_a.recv_size, 0);
		assert_eq!(channel_a.send_size, 0);
		assert_eq!(channel_a.recv_packet_count, 0);
		assert_eq!(channel_a.send_packet_count, 0);

		let now = Instant::now();
		let send_data = vec![1, 2, 3];
		channel_a
			.send_to(now, send_data.as_slice(), channel_b.socket.local_addr().unwrap())
			.unwrap();
		std::thread::sleep(Duration::from_millis(10));
		let mut recv_data = [0; 1024];
		channel_b.recv(now, &mut recv_data).unwrap();

		assert_eq!(channel_a.recv_size, 0);
		assert_eq!(channel_a.send_size, 3);
		assert_eq!(channel_a.recv_packet_count, 0);
		assert_eq!(channel_a.send_packet_count, 1);

		assert_eq!(channel_b.recv_size, 3);
		assert_eq!(channel_b.send_size, 0);
		assert_eq!(channel_b.recv_packet_count, 1);
		assert_eq!(channel_b.send_packet_count, 0);
	}
}
