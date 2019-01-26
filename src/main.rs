use daemon::Daemon;
use daemon::DaemonRunner;
use daemon::State;
use std::io::Error;
use std::sync::mpsc::Receiver;

fn main() -> Result<(), Error> {
	let daemon = Daemon {
		name: "clipable".to_string(),
	};
	
	daemon.run(move |rx: Receiver<State>| {
		for signal in rx.iter() {
			match signal {
				State::Start => starter(),
				State::Reload => reloader(),
				State::Stop => stopper(),
			};
		}
	})?;

	Ok(())
}

fn starter() {
	
}

fn reloader() {
	
}

fn stopper() {
	
}