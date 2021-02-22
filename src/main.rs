use std::mem;
use std::time::{Duration, Instant};
use slipstream::prelude::*;
use rayon::prelude::*;
use rand::random;
use multiversion::multiversion;
use structopt::StructOpt;

type C = usizex8;

#[multiversion]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "x86+sse")]
fn set(data: &mut Vec<C>, pat: usize) {
    data.par_iter_mut().enumerate().for_each(|(i, d)| {
        *d = C::splat(i ^ pat);
    })
}

#[multiversion]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "x86+sse")]
fn check(data: &Vec<C>, pat: usize) {
    data.par_iter().enumerate().for_each(|(i, d)| {
        assert_eq!(*d, C::splat(i ^ pat));
    })
}

#[multiversion]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "x86+sse")]
fn patt(data: &mut Vec<C>, offset: u32) {
    let mut val = C::default();
    val[offset as usize / 64] = 1 << (offset % 64);
    data.par_iter_mut().for_each(|d| {
        *d = val;
    });
    data.par_iter_mut().for_each(|d| {
        *d = val;
    });
    let neg = !val;
    data.par_iter_mut().for_each(|d| {
        assert_eq!(*d, val);
        *d = neg;
    });
    data.par_iter_mut().for_each(|d| {
        *d = neg;
    });
    data.par_iter().for_each(|d| {
        assert_eq!(*d, neg);
    })
}

#[derive(Debug, StructOpt)]
struct Opts {
    /// Size, in gigabytes.
    #[structopt(short = "s", long = "size", default_value = "64")]
    size: usize,
}

fn main() {
    let opts = Opts::from_args();
    let cnt = opts.size * 1024 * 1024 * 1024 / mem::size_of::<C>();
    let start = Instant::now();
    eprintln!("Start");
    let mut v = vec![C::default(); cnt];
    eprintln!("{:?} Alloc", start.elapsed());
    for i in 0..64 * 8 {
        patt(&mut v, i);
        eprintln!("{:?} Checked pattern with offset {}", start.elapsed(), i);
    }
    drop(v);
    let mut v1 = vec![C::default(); cnt / 2];
    let mut v2 = vec![C::default(); cnt / 2];
    eprintln!("{:?} Alloc", start.elapsed());
    set(&mut v1, 0);
    let mut pat = 0;
    eprintln!("{:?} Set", start.elapsed());
    for _ in 0..15 {
        check(&v1, pat);
        eprintln!("{:?} Check", start.elapsed());
        let inner = Instant::now();
        while inner.elapsed() < Duration::from_secs(1200) {
            eprint!(".");
            let r = random();
            set(&mut v2, r);
            check(&v2, r);
        }
        mem::swap(&mut v1, &mut v2);
        pat = random();
        set(&mut v1, pat);
    }
}
