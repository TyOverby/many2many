extern crate many2many;
use many2many::mchannel;

fn main() {
    let (sx, rx)= mchannel();

    let r1 = rx.clone();
    let r2 = r1.clone_filter(|&x| x % 2 == 0);
    let r3 = r2.clone_filter(|&x| x < 6);


    sx.send(5).unwrap();
    sx.send(10).unwrap();
    sx.send(2).unwrap();
    sx.send(0).unwrap();

    println!("{} {} {}",
             r1.recv().unwrap(),
             r2.recv().unwrap(),
             r3.recv().unwrap());
    println!("{} {} {}",
             r1.recv().unwrap(),
             r2.recv().unwrap(),
             r3.recv().unwrap());
}
