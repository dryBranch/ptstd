use std::thread::{JoinHandle, self};
use std::sync::{mpsc, Arc, Mutex};

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Stop,
}

/// 线程池
pub struct ThreadPool {
    workers     : Vec<Worker>,
    workers_len : usize,
    sender      : mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(max_worker: usize) -> ThreadPool {
        if max_worker == 0 {
            panic!("worker number should not be zero");
        }

        let (tx, rx) = mpsc::channel();
        let mut workers = Vec::with_capacity(max_worker);
        let receiver = Arc::new(Mutex::new(rx));
        
        for i in 0..max_worker {
            let worker = Worker::new(i, Arc::clone(&receiver));
            workers.push(worker);
        }

        ThreadPool {
            workers,
            workers_len: max_worker,
            sender: tx
        }
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Message::NewJob(Box::new(f));
        self.sender.send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // 给所有工作线程发送停止消息
        // 发送对应数量的停止信号
        for _ in 0..self.workers_len {
            let message = Message::Stop;
            self.sender.send(message).unwrap();
        }

        for w in &mut self.workers {
            if let Some(t) = w.thread.take() {
                t.join().unwrap();
            }
        }
    }
}

struct Worker {
    _id          : usize,
    thread      : Option<JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize, 
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>
    ) -> Worker {
        let t = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::NewJob(job) => {
                        job();
                    },
                    Message::Stop => break,
                }
            }
        });
        Worker { _id: id, thread: Some(t) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn sell(id: usize, ticket: Arc<Mutex<i32>>) {
        let mut n = 0;
        loop {
            let mut t = ticket.lock().unwrap();
            if *t < 10000 { 
                *t += 1;
                n += 1;
                println!("machine {}: ticket {} sold", id, *t)
            } else {
                break;
            }
        }
        println!("machine {} sold {}", id, n);
    }

    #[test]
    fn test1() {
        let cnt = Arc::new(Mutex::new(0));
        let max = 4;
        let pool = ThreadPool::new(max);
        for i in 0..max {
            let t = Arc::clone(&cnt);
            pool.execute(move || {
                sell(i, t);
            });
        }
    }
}
