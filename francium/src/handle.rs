use alloc::sync::Arc;
use alloc::boxed::Box;
use spin::Mutex;

use crate::process::Process;
use crate::memory::AddressSpace;

struct HandleError;

struct HandleObject<T> {
	obj: Arc<Mutex<Box<T>>>
}

impl<T> HandleObject<T> {
	fn new(x: Box<T>) -> HandleObject<T> {
		HandleObject{obj: Arc::new(Mutex::new(x))}
	}

	fn to_box(self) -> Box<T> {
		match Arc::try_unwrap(self.obj) {
			Ok(x) => x.into_inner(),
			Err(arc) => panic!("what did you do wtf")
		}
	}
}

enum Handle {
	Process(HandleObject<Process>),
	AddressSpace(HandleObject<AddressSpace>)
}