pub use kparse::*;

#[test]
fn test1() {
    let buf = "1234abcd";
    let t = &buf[4..];

    unsafe {
        assert_eq!(t.undo_subslice(4), buf);
    }
}

#[test]
fn test2() {}
