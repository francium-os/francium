use alloc::sync::Arc;
use alloc::boxed::Box;
use spin::Mutex;

use crate::process::Process;
use crate::memory::AddressSpace;
use crate::svc::ports::ServerPort;

struct HandleError;

pub struct HandleObject<T> {
	pub obj: Arc<Mutex<Box<T>>>
}

impl<T> HandleObject<T> {
	pub fn new(x: Box<T>) -> HandleObject<T> {
		HandleObject{obj: Arc::new(Mutex::new(x))}
	}

	fn to_box(self) -> Box<T> {
		match Arc::try_unwrap(self.obj) {
			Ok(x) => x.into_inner(),
			Err(arc) => panic!("what did you do wtf")
		}
	}
}

pub enum Handle {
	Process(HandleObject<Process>),
	AddressSpace(HandleObject<AddressSpace>)
}