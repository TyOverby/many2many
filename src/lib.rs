use std::task::spawn;
use std::comm::{channel, Sender, Receiver, Messages, TryRecvError};

pub struct MReceiver<T> {
    subscribe: Sender<Sender<T>>,
    rec: Receiver<T>
}

pub fn mchannel<T: Send + Clone>() -> (Sender<T>, MReceiver<T>) {
    let (ds, dr) = channel();

    (ds, MReceiver::from_receiver(dr))
}

fn listen<T: Send + Clone>(r: Receiver<T>) -> Sender<Sender<T>> {
    let (ls, lr) = channel();
    spawn(proc() {
        let dr = r;
        let lr = lr;

        let mut connected = vec![];
        loop {
            match dr.recv_opt() {
                Ok(m) => {
                    let m: T = m;
                    loop {
                        match lr.try_recv() {
                            Ok(l) => connected.push(l),
                            Err(_) => break
                        }
                    }
                    connected.retain(|l: &Sender<T>| {
                        match l.send_opt(m.clone()) {
                            Ok(()) => true,
                            Err(_) => false
                        }
                    });
                }
                Err(()) => break
            }
        }
    });
    ls
}

impl <T> MReceiver<T> where T: Send + Clone {
    pub fn from_receiver(r: Receiver<T>) -> MReceiver<T> {
        MReceiver::from_sub(listen(r))
    }

    fn from_sub(subscriber: Sender<Sender<T>>) -> MReceiver<T> {
        let (sx, rx)= channel();
        let _ = subscriber.send_opt(sx);
        MReceiver {
            subscribe: subscriber,
            rec: rx
        }
    }

    pub fn unwrap(self) -> Receiver<T> {
        self.rec
    }

    pub fn recv(&self) -> T {
        self.rec.recv()
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.rec.try_recv()
    }

    pub fn recv_opt(&self) -> Result<T, ()> {
        self.rec.recv_opt()
    }

    pub fn iter<'a>(&'a self) -> Messages<'a, T> {
        self.rec.iter()
    }
}

impl <T> Clone for MReceiver<T> where T: Send + Clone {
    fn clone(&self) -> MReceiver<T> {
        MReceiver::from_sub(self.subscribe.clone())
    }
}
