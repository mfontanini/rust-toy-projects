#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate crawler;
#[macro_use] extern crate log;
extern crate simple_logger;
extern crate url;

use std::env;
use std::process::exit;

use url::Url;

use crawler::worker::WorkerMaster;

use chan_signal::Signal;

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let args : Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <base-url>", args[0]);
        exit(1);
    }

    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);

    let starting_url = Url::parse(&args[1]).unwrap();
    let master = WorkerMaster::new(2);
    master.lock().unwrap().submit_url(starting_url);

    chan_select! {
        signal.recv() -> signal => {
            info!("Received signal: {:?}, stopping", signal.unwrap())
        }
    }
}
