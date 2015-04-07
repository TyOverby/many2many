use std::thread::spawn;
use std::sync::mpsc::{channel, Sender, Receiver, RecvError, TryRecvError, Iter};
use std::convert::From;

pub struct MReceiver<T> {
    subscribe: Sender<Sender<T>>,
    rec: Receiver<T>
}

pub fn mchannel<T: Send + Clone + 'static>() -> (Sender<T>, MReceiver<T>) {
    let (ds, dr) = channel();

    (ds, MReceiver::from_receiver(dr))
}

fn listen<T: Send + Clone + 'static>(r: Receiver<T>) -> Sender<Sender<T>> {
    let (ls, lr) = channel();
    spawn(move || {
        let dr = r;
        let lr = lr;

        let mut connected = vec![];
        loop {
            match dr.recv() {
                Ok(m) => {
                    let m: T = m;
                    loop {
                        match lr.try_recv() {
                            Ok(l) => connected.push(l),
                            Err(_) => break
                        }
                    }

                    connected.retain(|l: &Sender<T>| {
                        match l.send(m.clone()) {
                            Ok(()) => true,
                            Err(_) => false
                        }
                    });
                }
                Err(_) => break
            }
        }
    });
    ls
}

impl <T> From<Receiver<T>> for MReceiver<T> where T: Send + Clone + 'static {
    fn from(other: Receiver<T>) -> MReceiver<T> {
        MReceiver::from_sub(listen(other))
    }
}

impl <T> MReceiver<T> where T: Send + Clone + 'static {
    fn from_receiver(other: Receiver<T>) -> MReceiver<T> {
        MReceiver::from_sub(listen(other))
    }

    fn from_sub(subscriber: Sender<Sender<T>>) -> MReceiver<T> {
        let (sx, rx)= channel();
        let _ = subscriber.send(sx);
        MReceiver {
            subscribe: subscriber,
            rec: rx
        }
    }

    pub fn unwrap(self) -> Receiver<T> {
        self.rec
    }

    pub fn recv(&self) -> Result<T, RecvError> {
        self.rec.recv()
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.rec.try_recv()
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        self.rec.iter()
    }
}

impl <T> Clone for MReceiver<T> where T: Send + Clone + 'static {
    fn clone(&self) -> MReceiver<T> {
        MReceiver::from_sub(self.subscribe.clone())
    }
}
