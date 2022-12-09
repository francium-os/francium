use tracing::Subscriber;
use tracing::{Metadata, span::Attributes, span::Record, Id, Event, field::{Value, Visit, Field}};

pub fn init() {
	use tracing_subscriber::util::SubscriberInitExt;

    let coll = LogSubscriber{};
    coll.try_init().expect("Registering subscriber failed");

	//unimplemented!();
}

struct LogSubscriber {
	// todo
}

pub struct PrintVisitor {
}

impl Visit for PrintVisitor {
	fn record_debug(&mut self, field: &Field, value: &dyn core::fmt::Debug) {
		print!("{} = {:?}; ", field.name(), value);
	}
}

// Default impls to override:
// register_callsite
// event_enabled
// clone_span
// try_close

impl Subscriber for LogSubscriber {
	// todo
	fn enabled(&self, metadata: &Metadata<'_>) -> bool {
		//println!("meta: {:?}", metadata);
		true
	}

	fn new_span(&self, _: &Attributes<'_>) -> Id {
		todo!()
	}

	fn record(&self, _: &Id, _: &Record<'_>) {
		todo!()
	}
	
	fn record_follows_from(&self, _: &Id, _: &Id) {
		todo!()
	}
	
	fn event(&self, event: &Event<'_>) {
		//println!("Event {:?}", event);
		event.record(&mut PrintVisitor{});
		println!();
	}
	
	fn enter(&self, _: &Id){
		todo!()
	}

	fn exit(&self, _: &Id) {
		todo!()
	}
}