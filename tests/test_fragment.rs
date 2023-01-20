pub use kparse::*;

// #[test]
// fn test_union_of() {
//     let tt = "iiii";
//     let a = "eeee";
//     let b = "gggg";
//
//     let u = tt.union_of(&a[0..0], &a[0..0]);
//
//     println!("{:?}", u)
// }

#[test]
fn test_current_prefix() {
    fn run(buf: &str, nl: &[usize]) {
        for i in 0..=buf.len() {
            for j in i..=buf.len() {
                let test = &buf[i..j];
                let prefix = buf.current_prefix(test, b'\n');

                let mut cmp_start = 0;
                let mut cmp_end = 0;
                for idx in nl {
                    if i <= *idx {
                        cmp_end = i;
                        break;
                    } else {
                        cmp_start = *idx + 1;
                    }
                }

                let cmp = &buf[cmp_start..cmp_end];

                println!("{} {} : {:?} {:?} -> {}", i, j, test, cmp, prefix);

                assert_eq!(prefix, cmp);
            }
        }
    }

    run("1234\n5678\n90ab", &[4, 9, 14]);
    run("1234\n5678\n90ab\n", &[4, 9, 14, 15]);
}

#[test]
fn test_subslice_offset() {
    let a = "1234";
    let b = "5678";

    // UB: wrong origin of b.
    // but that's about the only thing we can't catch.
    unsafe {
        println!("{}", a.subslice_offset(b));
    }
}

#[test]
#[should_panic]
fn test_subslice_offset_2() {
    //              01234 56789 01234
    let buf = "1234\n5678\n90ab";
    let f0 = &buf[..8];
    let f1 = &buf[10..];

    unsafe {
        println!("{}", f0.subslice_offset(f1));
    }
}

#[test]
fn test_subslice_offset_3() {
    //              01234 56789 01234
    let buf = "1234\n5678\n90ab";
    let f0 = &buf[..8];
    let f1 = &buf[8..];

    unsafe {
        println!("{}", f0.subslice_offset(f1));
    }
}

#[test]
fn test_und_subslice_1() {
    let buf = "1234abcd";
    let t = &buf[4..];

    unsafe {
        assert_eq!(t.undo_subslice(4), buf);
    }
}
