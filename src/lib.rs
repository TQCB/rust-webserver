use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Message type for communicating with worker threads.
/// 
/// This enum allows for explicit control over worker lifecycle,
/// enabling graceful shutdown by sending Terminate messages.
enum Message {
    /// A new job to be executed by the worker.
    NewJob(Job),
    /// Signal to the worker to terminate gracefully.
    Terminate,
}

/// A thread pool for executing jobs concurrently.
///
/// The ThreadPool manages a fixed number of worker threads that
/// process jobs from a shared queue. When the pool is dropped,
/// it sends termination signals to all workers and waits for them
/// to finish their current jobs before shutting down.
///
/// # Example
///
/// ```no_run
/// use webserver::ThreadPool;
///
/// let pool = ThreadPool::new(4);
/// pool.execute(|| {
///     println!("Job executed!");
/// });
/// ```
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Message>>,
}

impl ThreadPool {
    /// Creates a new ThreadPool.
    ///
    /// The `size` parameter specifies the number of worker threads
    /// in the pool. Each worker thread will process jobs from a
    /// shared queue.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of worker threads to create
    ///
    /// # Panics
    ///
    /// The `new` function will panic if `size` is zero.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use webserver::ThreadPool;
    ///
    /// let pool = ThreadPool::new(4);
    /// ```
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&rx)))
        }

        ThreadPool {
            workers,
            sender: Some(tx),
        }
    }

    /// Executes a closure on one of the worker threads.
    ///
    /// The closure will be executed asynchronously by one of the
    /// available worker threads. If all workers are busy, the job
    /// will be queued until a worker becomes available.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that takes no arguments and returns nothing.
    ///          The closure must be `Send` and `'static`.
    ///
    /// # Errors
    ///
    /// This function will panic if the sender channel has been closed,
    /// which typically only happens during shutdown.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use webserver::ThreadPool;
    ///
    /// let pool = ThreadPool::new(4);
    /// pool.execute(|| {
    ///     println!("This runs on a worker thread");
    /// });
    /// ```
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let message = Message::NewJob(job);
        
        if let Some(sender) = self.sender.as_ref() {
            sender
                .send(message)
                .expect("Should've been able to send job to worker");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Send explicit terminate messages to all workers
        if let Some(sender) = self.sender.take() {
            for _ in &self.workers {
                let _ = sender.send(Message::Terminate);
            }
        }

        // Wait for all workers to finish
        for worker in self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);
            if let Err(e) = worker.thread.join() {
                eprintln!("Error joining worker {}: {:?}", worker.id, e);
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        println!("Creating worker {id}");
        let thread = thread::spawn(move || {
            loop {
                let message = {
                    let receiver_guard = receiver.lock();
                    match receiver_guard {
                        Ok(guard) => guard.recv(),
                        Err(e) => {
                            eprintln!("Worker {id} failed to acquire lock: {:?}", e);
                            break;
                        }
                    }
                };

                match message {
                    Ok(Message::NewJob(job)) => {
                        println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Ok(Message::Terminate) => {
                        println!("Worker {id} received terminate signal, shutting down.");
                        break;
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected, shutting down.");
                        break;
                    }
                }
            }
        });
        Worker { id, thread }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_thread_pool_executes_job() {
        let pool = ThreadPool::new(2);
        let counter = Arc::new(Mutex::new(0));

        let counter_clone = Arc::clone(&counter);
        pool.execute(move || {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
        });

        // Give the worker thread time to execute
        thread::sleep(Duration::from_millis(100));

        let count = counter.lock().unwrap();
        assert_eq!(*count, 1);
    }

    #[test]
    fn test_thread_pool_executes_multiple_jobs() {
        let pool = ThreadPool::new(2);
        let counter = Arc::new(Mutex::new(0));

        for _ in 0..5 {
            let counter_clone = Arc::clone(&counter);
            pool.execute(move || {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
            });
        }

        // Give worker threads time to execute all jobs
        thread::sleep(Duration::from_millis(200));

        let count = counter.lock().unwrap();
        assert_eq!(*count, 5);
    }

    #[test]
    fn test_thread_pool_graceful_shutdown() {
        let pool = ThreadPool::new(2);
        let counter = Arc::new(Mutex::new(0));

        let counter_clone = Arc::clone(&counter);
        pool.execute(move || {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
        });

        // Drop the pool, which should trigger graceful shutdown
        drop(pool);

        // Verify the job was executed
        let count = counter.lock().unwrap();
        assert_eq!(*count, 1);
    }
}
