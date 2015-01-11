use super::mchannel;

#[test]
fn test_basic() {
    let (sx, rx)= mchannel();

    // Multiple receivers!
    let r1 = rx.clone();
    let r2 = rx.clone();
    let r3 = rx.clone();

    sx.send(5u32);
    sx.send(6u32);

    // They all get the same messages.
    assert!(r1.recv() == 5);
    assert!(r2.recv() == 5);
    assert!(r3.recv() == 5);

    assert!(r1.recv() == 6);
    assert!(r2.recv() == 6);
    assert!(r3.recv() == 6);
}

