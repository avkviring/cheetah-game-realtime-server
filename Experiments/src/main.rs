use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;

struct Struct<'a> {
	pub listener: SomeListener,
	pub vec: Vec<&'a dyn Listener>,
}

trait Listener {
	fn do_some(&mut self);
}

impl Struct<'_> {
	fn new() -> Self {
		let mut result = Self {
			listener: SomeListener {},
			vec: vec![],
		};
		result.vec.push(&result.listener);
		result
	}
}

struct SomeListener {}

impl Listener for SomeListener {
	fn do_some(&mut self) {}
}


fn main() {
	let listener = RefCell::new(SomeListener {});
	let mut s = Struct {
		listener: listener.clone(),
		vec: vec![listener],
	};
	
	s.vec[0].borrow_mut().do_some();
}
