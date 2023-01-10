use kparse::{DataFrames, StrLines};
use std::arch::x86_64::_rdrand16_step;
use std::hint::black_box;

#[test]
#[should_panic]
fn test_this_stack() {
    unsafe {
        let zero = "zero";
        let one = "one";
        let two = "two";

        let frames = StrLines::new(zero);

        dbg!(frames.start(zero));
        dbg!(frames.start(one));
        dbg!(frames.start(two));
    }
}

#[test]
#[should_panic]
fn test_this() {
    unsafe {
        let some_string = String::from("abcd");

        // do some memory shuffling via the allocator
        let mut v = Vec::new();
        for i in 0..10000 {
            let mut rand = 0u16;
            if 1 == _rdrand16_step(&mut rand) {
                v.push(String::from("x".repeat(rand as usize)));

                if 0 == i % 5 {
                    v.remove(rand as usize % v.len());
                }
            } else {
                panic!("_rdrand16_step failed?");
            }
        }
        black_box(&v);

        let frames = StrLines::new(&some_string);
        // choose one
        for _ in 0..3000 {
            let mut rand = 0u16;
            let frag = if 1 == _rdrand16_step(&mut rand) {
                v.get(rand as usize % v.len()).unwrap()
            } else {
                panic!("_rdrand16_step failed?");
            };

            // should panic.
            dbg!(frames.start(&frag));
        }
    }
}

#[test]
fn test_iter() {
    let run = |buf: &str, r: &[&str]| {
        let frames = StrLines::new(buf);
        let mut it = frames.iter();
        let mut jt = r.iter();

        // print!("{:?} ->", buf);
        loop {
            let i = it.next();
            let j = jt.next().copied();
            assert_eq!(i, j);
            // print!("{:?} ", i);

            if i.is_none() && j.is_none() {
                break;
            }
        }

        // println!();
    };

    let buf = "1234";
    run(buf, &["1234"]);
    let buf = "1234\n5678\n90ab";
    run(buf, &["1234", "5678", "90ab"]);
    let buf = "1234\n5678\n90ab\n";
    run(buf, &["1234", "5678", "90ab", ""]);
    let buf = "\n1234\n5678\n90ab";
    run(buf, &["", "1234", "5678", "90ab"]);
    let buf = "1234\n5678\n90ab\ncdef\nghij";
    run(buf, &["1234", "5678", "90ab", "cdef", "ghij"]);
    let buf = "\n1234\n5678\n90ab\ncdef\nghij";
    run(buf, &["", "1234", "5678", "90ab", "cdef", "ghij"]);
    let buf = "1234\n5678\n90ab\ncdef\nghij\n";
    run(buf, &["1234", "5678", "90ab", "cdef", "ghij", ""]);
}

#[test]
fn test_str_simple() {
    unsafe {
        let buf = "1234\n5678\n90ab";
        let frames = StrLines::new(buf);

        let run = |frag: &str| {
            // println!(
            //     "'{}'-> '{}' @ {}",
            //     frag,
            //     frames.current(frag),
            //     frames.offset(frag)
            // );

            let mut it = frames.forward_from(frag);
            it.next();
            it.next();
            it.next();
            it.next();
            // print!("forward_from {:?} ", it.next());
            // print!("{:?} ", it.next());
            // print!("{:?} ", it.next());
            // println!();
            let mut it = frames.backward_from(frag);
            it.next();
            it.next();
            it.next();
            it.next();
            // print!("backward_from {:?} ", it.next());
            // print!("{:?} ", it.next());
            // print!("{:?} ", it.next());
            // println!();
        };

        for s in 0..=14 {
            for e in s..=14 {
                let frag = &buf[s..e];
                run(frag);
            }
        }
    }
}

fn find_idx(p: usize, pos_nl: &[usize]) -> usize {
    // print!("find {} in {:?}", p, pos_nl);
    // find the right idx into split
    let mut it_p = pos_nl.iter().enumerate();
    let idx = loop {
        let Some((idx, pos)) = it_p.next() else {
                panic!("x not found");
            };
        if p <= *pos {
            break idx;
        }
    };
    // println!(" -> {}", idx);
    idx
}

fn cmp_it<'a>(mut it: impl Iterator<Item = &'a str>, mut jt: impl Iterator<Item = &'a str>) {
    loop {
        let i = it.next();
        let j = jt.next();

        assert_eq!(i, j);
        // println!("cmp_it {:?} == {:?}", i, j);

        if i.is_none() && j.is_none() {
            break;
        }
    }
}

#[test]
fn test_str_full() {
    // assume groups of 4
    fn run(buf: &str, pos_nl: &[usize], split: &[&str]) {
        let frames = StrLines::new(buf);

        for s in 0..=buf.len() {
            let i_s = find_idx(s, pos_nl);

            for e in s..=buf.len() {
                let i_e = find_idx(e, pos_nl);

                let frag = &buf[s..e];

                // println!();
                // println!(
                //     "find {:?} in {:?} with start {}, end {}, offset {}",
                //     frag,
                //     buf,
                //     i_s,
                //     i_e,
                //     unsafe { offset_str(buf, frag) }
                // );

                // println!("current:");
                let it = unsafe { frames.current(frag) };
                let jt = split[i_s..i_e + 1].iter().copied();
                cmp_it(it, jt);

                // println!("forward:");
                let it = unsafe { frames.forward_from(frag) };
                let jt = split[i_e + 1..].iter().copied();
                cmp_it(it, jt);

                // println!("backward:");
                let it = unsafe { frames.backward_from(frag) };
                let jt = split[..i_s].iter().rev().copied();
                cmp_it(it, jt);
            }
        }
    }

    //   01234
    run("1234", &[4], &["1234"]);
    //   01234 56789 01234
    run("1234\n5678\n90ab", &[4, 9, 14], &["1234", "5678", "90ab"]);
    //   01234 56789 01234 5
    run(
        "1234\n5678\n90ab\n",
        &[4, 9, 14, 15],
        &["1234", "5678", "90ab", ""],
    );
    //   0 12345 67890 12345
    run(
        "\n1234\n5678\n90ab",
        &[0, 5, 10, 15],
        &["", "1234", "5678", "90ab"],
    );
    //   01234 56789 01234 56789 01234
    run(
        "1234\n5678\n90ab\ncdef\nghij",
        &[4, 9, 14, 19, 24],
        &["1234", "5678", "90ab", "cdef", "ghij"],
    );
    //   0 12345 67890 12345 67890 12345
    run(
        "\n1234\n5678\n90ab\ncdef\nghij",
        &[0, 5, 10, 15, 20, 25],
        &["", "1234", "5678", "90ab", "cdef", "ghij"],
    );
    //   01234 56789 01234 56789 01234 5
    run(
        "1234\n5678\n90ab\ncdef\nghij\n",
        &[4, 9, 14, 19, 24, 25],
        &["1234", "5678", "90ab", "cdef", "ghij", ""],
    );
}

#[test]
fn test_current() {
    fn run(buf: &str, pos_nl: &[usize], split: &[&str]) {
        let frames = StrLines::new(buf);

        for s in 0..=buf.len() {
            let i_s = find_idx(s, pos_nl);

            for e in s..=buf.len() {
                let i_e = find_idx(e, pos_nl);

                let frag = &buf[s..e];

                // println!();
                // println!(
                //     "find {:?} in {:?} with start {}, end {}",
                //     frag, buf, i_s, i_e
                // );

                let start = unsafe { frames.start(frag) };
                assert_eq!(start, split[i_s]);
                let end = unsafe { frames.end(frag) };
                assert_eq!(end, split[i_e]);
            }
        }
    }

    //   01234
    run("1234", &[4], &["1234"]);
    //   01234 56789 01234
    run("1234\n5678\n90ab", &[4, 9, 14], &["1234", "5678", "90ab"]);
    //   01234 56789 01234 5
    run(
        "1234\n5678\n90ab\n",
        &[4, 9, 14, 15],
        &["1234", "5678", "90ab", ""],
    );
    //   0 12345 67890 12345
    run(
        "\n1234\n5678\n90ab",
        &[0, 5, 10, 15],
        &["", "1234", "5678", "90ab"],
    );
    //   01234 56789 01234 56789 01234
    run(
        "1234\n5678\n90ab\ncdef\nghij",
        &[4, 9, 14, 19, 24],
        &["1234", "5678", "90ab", "cdef", "ghij"],
    );
    //   0 12345 67890 12345 67890 12345
    run(
        "\n1234\n5678\n90ab\ncdef\nghij",
        &[0, 5, 10, 15, 20, 25],
        &["", "1234", "5678", "90ab", "cdef", "ghij"],
    );
    //   01234 56789 01234 56789 01234 5
    run(
        "1234\n5678\n90ab\ncdef\nghij\n",
        &[4, 9, 14, 19, 24, 25],
        &["1234", "5678", "90ab", "cdef", "ghij", ""],
    );
}
