extern crate many2many;
use many2many::{MReceiver, mchannel};

fn main() {
    let (sx, rx)= mchannel();

    let r1 = rx.clone();
    let r2 = rx.clone();
    let r3 = rx.clone();

    sx.send(5u32);

    println!("{} {} {}",
             r1.recv().unwrap(),
             r2.recv().unwrap(),
             r3.recv().unwrap());
}
