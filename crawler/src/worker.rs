use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Weak;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;

use url::Url;

use reqwest;

use parser::LinkParser;

enum WorkerMessage {
    Work(Url),
}

type ReceiverChannel = Arc<Mutex<mpsc::Receiver<WorkerMessage>>>;

pub struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

pub struct WorkerMaster {
    seen_urls : Mutex<HashSet<Url>>,
    workers : Vec<Worker>,
    workers_channel : Mutex<mpsc::Sender<WorkerMessage>>
}

impl WorkerMaster {
    pub fn new(worker_count: u32) -> Arc<Mutex<WorkerMaster>> {
        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        let master = Arc::new(Mutex::new(WorkerMaster {
            seen_urls: Mutex::new(HashSet::new()),
            workers: Vec::new(),
            workers_channel: Mutex::new(tx)
        }));
        for _ in 0..worker_count {
            let weak_master = Arc::downgrade(&master);
            master.lock().unwrap().workers.push(Worker::new(Arc::clone(&rx), weak_master));
        }
        master
    }

    pub fn submit_url(&mut self, url: Url) {
        let mut seen_urls = self.seen_urls.lock().unwrap();
        if seen_urls.insert(url.clone()) {
            info!("Submitting URL {}", url);
            self.workers_channel.lock().unwrap().send(WorkerMessage::Work(url)).unwrap();
        }
    }
}

impl Drop for WorkerMaster {
    fn drop(&mut self) {
        for w in &mut self.workers {
            if let Some(t) = w.thread.take() {
                t.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(channel: ReceiverChannel, master_weak: Weak<Mutex<WorkerMaster>>) -> Worker {
        let handler = move || {
            let mut client = reqwest::Client::new();
            loop {
                match channel.lock().unwrap().recv().unwrap() {
                    WorkerMessage::Work(url) => {
                        info!("Got URL to process: {:?}", url);
                        let result = Worker::process_url(&mut client, url);
                        match result {
                            Ok(urls) => {
                                let master = master_weak.upgrade();
                                if let Some(master) = master {
                                    urls.into_iter().for_each(|url| {
                                        master.lock().unwrap().submit_url(url);
                                    });
                                }
                                else {
                                    info!("Stopping because the master is gone");
                                    break
                                }
                            }
                            Err(error) => {
                                warn!("Error while processing URL: {}", error);
                            }
                        }
                    }
                }
            }
        };
        Worker {
            thread: Some(thread::spawn(handler)),
        }
    }

    fn process_url(client: &mut reqwest::Client, url: Url) -> Result<Vec<Url>, reqwest::Error> {
        info!("Fetching URL {}", url);

        let body = client.get(url.clone())
                    .send()?
                    .text()?;
        let parser = LinkParser::new(url);
        Ok(parser.parse(&body))
    }
}
