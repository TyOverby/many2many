use std::thread::spawn;
use std::sync::mpsc::{channel, Sender, Receiver, RecvError, TryRecvError, Iter};
use std::convert::From;
use std::sync::Arc;

type FilterFn<T> = Option<Arc<Box<Fn(&T) -> bool + 'static + Sync + Send>>>;

pub struct MReceiver<T> {
    subscribe: Sender<(Sender<T>, FilterFn<T>)>,
    rec: Receiver<T>,
    filter: FilterFn<T>
}

pub fn mchannel<T: Send + Clone + 'static>() -> (Sender<T>, MReceiver<T>) {
    let (ds, dr) = channel();

    (ds, MReceiver::from_receiver(dr, None))
}

fn listen<T: Send + Clone + 'static>(r: Receiver<T>) -> Sender<(Sender<T>, FilterFn<T>)> {
    let (ls, lr) = channel();
    spawn(move || {
        let dr = r;
        let lr = lr;

        let mut connected = vec![];
        while let Ok(m) = dr.recv() {
            let m: T = m;
            while let Ok(l) = lr.try_recv() {
                connected.push(l)
            }

            connected.retain(|out| {
                let &(ref l, ref f): &(Sender<T>, FilterFn<T>) = out;
                let f: &FilterFn<T> = f;

                if f.as_ref().map(|f| f(&m)).unwrap_or(true) {
                    l.send(m.clone()).is_ok()
                } else {
                    true
                }
            });
        }
    });
    ls
}

impl <T> From<Receiver<T>> for MReceiver<T> where T: Send + Clone + 'static {
    fn from(other: Receiver<T>) -> MReceiver<T> {
        MReceiver::from_sub(listen(other), None)
    }
}

impl <T> MReceiver<T> where T: Send + Clone + 'static {
    fn from_receiver(other: Receiver<T>, filter: FilterFn<T>) -> MReceiver<T> {
        MReceiver::from_sub(listen(other), filter)
    }

    fn from_sub(subscriber: Sender<(Sender<T>, FilterFn<T>)>,
                filter: FilterFn<T>) -> MReceiver<T> {
        let (sx, rx)= channel();
        let _ = subscriber.send((sx, filter.clone()));
        MReceiver {
            subscribe: subscriber,
            rec: rx,
            filter: filter
        }
    }

    pub fn into_inner(self) -> Receiver<T> {
        self.rec
    }

    pub fn as_inner(&self) -> &Receiver<T> {
        &self.rec
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

    pub fn clone_filter<F>(&self, f: F) -> MReceiver<T>
    where F: Fn(&T) -> bool + 'static + Send + Sync
    {
        if let Some(cur_fn) = self.filter.as_ref() {
            let old_f = cur_fn.clone();
            let new_f = move |a: &T| {
                old_f(a) && f(a)
            };

            MReceiver::from_sub(
                self.subscribe.clone(),
                Some(Arc::new(
                     Box::new(new_f) as Box<Fn(&T) -> bool + 'static + Send + Sync>)))
        } else {
            MReceiver::from_sub(
                self.subscribe.clone(),
                Some(Arc::new(
                     Box::new(f) as Box<Fn(&T) -> bool + 'static + Send + Sync>)))
        }
    }
}

impl <T> Clone for MReceiver<T> where T: Send + Clone + 'static {
    fn clone(&self) -> MReceiver<T> {
        MReceiver::from_sub(self.subscribe.clone(), None)
    }
}
