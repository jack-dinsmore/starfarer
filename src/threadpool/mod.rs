#![allow(dead_code)]

use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

type Job = Box<dyn FnBox + Send + 'static>;

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

enum Message {
    NewJob(Job),
    Terminate
}

pub struct ThreadPool {
    threads: Vec<Worker>,
    sender: Sender<Message>,
}

impl ThreadPool {
    pub fn new(num_workers: usize) -> Self {
        let mut threads = Vec::with_capacity(num_workers);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..num_workers {
            threads.push(Worker::new(id, Arc::clone(&receiver)))
        }

        Self {
            threads,
            sender,
        }
    }

    pub fn execute<T: Send + 'static, F: FnOnce() -> T + Send + 'static>(&self, f: F) -> Receiver<T> {
        let (sender, receiver) = mpsc::channel();
        self.sender.send(Message::NewJob(Box::new(move || { sender.send(f()).unwrap(); }))).unwrap();
        receiver
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.threads {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.threads {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::NewJob(job) => job.call_box(),
                    Message::Terminate => break,
                };
            }
        });
        Self {
            id,
            thread: Some(thread),
        }
    }
}