use std::thread;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::collections::VecDeque;

pub type MicaTask<T> = Box<dyn FnOnce() -> T + Send + 'static>;

pub struct Pool<T>
where
    T: Send + 'static,
    // F: FnOnce() -> T + Send + 'static
{
    queue: Mutex<VecDeque<MicaTask<T>>>,
    jobs_available: Condvar,
}

impl<T> Pool<T>
where
    T: Send + 'static,
    // F: FnOnce() -> T + Send + 'static
{
    pub fn new() -> Self {
        Pool {
            queue: Mutex::new(VecDeque::new()),
            jobs_available: Condvar::new(),
        }
    }

    pub fn submit(self: Arc<Self>, task: MicaTask<T>) {
        self.queue.lock().unwrap().push_back(task);
        self.jobs_available.notify_one();
    }

    pub fn init(self: Arc<Self>, num_threads: usize) -> Receiver<T> {
        let (tx, rx) = mpsc::channel::<T>();

        for _ in 0..num_threads {
            let pool = Arc::clone(&self);
            let tx = tx.clone();

            thread::spawn(move ||{
                loop {
                    let task = {
                        let mut q = pool.queue.lock().unwrap();
                        q = pool.jobs_available.wait(q).unwrap();
                        q.pop_front()
                    };
                    println!("Thread woken up");

                    if let Some(t) = task {
                        let result = t();
                        println!("Sending...");
                        tx.send(result).unwrap();
                    }
                }
            });
        }

        rx
    }
}

fn ex() {
    let thread = thread::spawn(|| 0);
}