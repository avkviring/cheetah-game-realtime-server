use serde::{Deserialize, Serialize};

///
/// Прикладные команды
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct ApplicationCommands {
	///
	/// С гарантией доставки
	///
	pub reliability: Vec<ApplicationCommand>,
	
	///
	/// Без гарантии доставки
	///
	pub unreliability: Vec<ApplicationCommand>,
	
}

impl ApplicationCommands {
	pub fn add(&mut self, command: &Self) {
		self.reliability.extend_from_slice(&command.reliability);
		self.unreliability.extend_from_slice(&command.unreliability);
	}
	
	pub fn clear(&mut self) {
		self.reliability.clear();
		self.unreliability.clear();
	}
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ApplicationCommand {
	Ping(String)
}