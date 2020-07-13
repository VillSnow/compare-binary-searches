#![feature(test)]
extern crate test;

use std::cmp::{Ord, Ordering, Ordering::*, PartialEq, PartialOrd};
use std::result::Result;
use std::thread::sleep;
use std::time::Duration;
use test::bench::Bencher;

#[derive(Eq, Clone, Copy)]
struct Slow {
    value: i32,
}

impl Slow {
    pub fn new(value: i32) -> Slow {
        Slow { value }
    }
}

impl PartialEq for Slow {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Equal
    }
}

impl Ord for Slow {
    fn cmp(&self, other: &Self) -> Ordering {
        eprintln!("Compare {} vs {}", self.value, other.value);
        sleep(Duration::from_millis(1));
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for Slow {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn logic1<T, F>(slice: &[T], mut f: F) -> Result<usize, usize>
where
    F: FnMut(&T) -> Ordering,
{
    let mut base = 0usize;
    let mut s = slice;

    loop {
        let (head, tail) = s.split_at(s.len() >> 1);
        if tail.is_empty() {
            return Err(base);
        }
        match f(&tail[0]) {
            Less => {
                base += head.len() + 1;
                s = &tail[1..];
            }
            Greater => s = head,
            Equal => return Ok(base + head.len()),
        }
    }
}

fn logic2<T, F>(slice: &[T], mut f: F) -> Result<usize, usize>
where
    F: FnMut(&T) -> Ordering,
{
    let s = slice;
    let mut size = s.len();
    if size == 0 {
        return Err(0);
    }
    let mut base = 0usize;
    while size > 1 {
        let half = size / 2;
        let mid = base + half;
        let cmp = f(unsafe { s.get_unchecked(mid) });
        base = if cmp == Greater { base } else { mid };
        size -= half;
    }
    let cmp = f(unsafe { s.get_unchecked(base) });
    if cmp == Equal {
        Ok(base)
    } else {
        Err(base + (cmp == Less) as usize)
    }
}

fn logic3<T, P>(slice: &[T], mut pred: P) -> usize
where
    P: FnMut(&T) -> bool,
{
    let mut left = 0;
    let mut right = slice.len();

    while left != right {
        let mid = left + (right - left) / 2;
        let value = unsafe { slice.get_unchecked(mid) };
        if pred(value) {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    left
}

fn logic4<T, P>(slice: &[T], mut pred: P) -> usize
where
    P: FnMut(&T) -> bool,
{
    let mut left = 0;
    let mut right = slice.len();

    while left != right {
        let mid = left + (right - left) / 2;
        //let value = unsafe { slice.get_unchecked(mid) };
        let is_ok = pred(unsafe { slice.get_unchecked(mid) });
        left = if is_ok { mid + 1 } else { left };
        right = if is_ok { right } else { mid };
    }

    left
}

fn partition_point_logic1<T, P>(slice: &[T], mut pred: P) -> usize
where
    P: FnMut(&T) -> bool,
{
    logic1(slice, |x| if pred(x) { Less } else { Greater }).unwrap_or_else(|i| i)
}

fn partition_point_logic2<T, P>(slice: &[T], mut pred: P) -> usize
where
    P: FnMut(&T) -> bool,
{
    logic2(slice, |x| if pred(x) { Less } else { Greater }).unwrap_or_else(|i| i)
}

fn partition_point_logic3<T, P>(slice: &[T], mut pred: P) -> usize
where
    P: FnMut(&T) -> bool,
{
    logic3(slice, pred)
}

fn partition_point_logic4<T, P>(slice: &[T], mut pred: P) -> usize
where
    P: FnMut(&T) -> bool,
{
    logic4(slice, pred)
}

fn main() {
    eprintln!("# search -1");
    check_number_of_comparison(-1);

    eprintln!("# search 0");
    check_number_of_comparison(1);

    eprintln!("# search 6");
    check_number_of_comparison(6);

    eprintln!("# search 7");
    check_number_of_comparison(7);
}

const LARGE_SLICE_SIZE: i32 = 65536;
const NOT_ALIGNED_SIZE: i32 = 7; // not 2^n

fn check_number_of_comparison(value: i32) {
    let ary = (0..NOT_ALIGNED_SIZE).map(Slow::new).collect::<Vec<_>>();
    eprintln!("logic1:");
    let n = partition_point_logic1(&ary, |&x| x < Slow::new(value));
    eprintln!("result: {}", n);
    eprintln!();

    eprintln!("logic2:");
    let n = partition_point_logic2(&ary, |&x| x < Slow::new(value));
    eprintln!("result: {}", n);
    eprintln!();

    eprintln!("logic3:");
    let n = partition_point_logic3(&ary, |&x| x < Slow::new(value));
    eprintln!("result: {}", n);
    eprintln!();

    eprintln!("logic4:");
    let n = partition_point_logic3(&ary, |&x| x < Slow::new(value));
    eprintln!("result: {}", n);
    eprintln!();
}

#[bench]
fn bench_partition_point_logic1_int(b: &mut Bencher) {
    let ary = (0..LARGE_SLICE_SIZE).collect::<Vec<_>>();
    b.iter(|| partition_point_logic1(&ary, |&x| x < 0));
}

#[bench]
fn bench_partition_point_logic2_int(b: &mut Bencher) {
    let ary = (0..LARGE_SLICE_SIZE).collect::<Vec<_>>();
    b.iter(|| partition_point_logic2(&ary, |&x| x < 0));
}

#[bench]
fn bench_partition_point_logic3_int(b: &mut Bencher) {
    let ary = (0..LARGE_SLICE_SIZE).collect::<Vec<_>>();
    b.iter(|| partition_point_logic3(&ary, |&x| x < 0));
}

#[bench]
fn bench_partition_point_logic4_int(b: &mut Bencher) {
    let ary = (0..LARGE_SLICE_SIZE).collect::<Vec<_>>();
    b.iter(|| partition_point_logic4(&ary, |&x| x < 0));
}

#[bench]
fn bench_partition_point_logic1_slow(b: &mut Bencher) {
    let ary = (0..NOT_ALIGNED_SIZE).map(Slow::new).collect::<Vec<_>>();
    b.iter(|| partition_point_logic1(&ary, |&x| x < Slow::new(0)));
}

#[bench]
fn bench_partition_point_logic2_slow(b: &mut Bencher) {
    let ary = (0..NOT_ALIGNED_SIZE).map(Slow::new).collect::<Vec<_>>();
    b.iter(|| partition_point_logic2(&ary, |&x| x < Slow::new(0)));
}

#[bench]
fn bench_partition_point_logic3_slow(b: &mut Bencher) {
    let ary = (0..NOT_ALIGNED_SIZE).map(Slow::new).collect::<Vec<_>>();
    b.iter(|| partition_point_logic3(&ary, |&x| x < Slow::new(0)));
}

#[bench]
fn bench_partition_point_logic4_slow(b: &mut Bencher) {
    let ary = (0..NOT_ALIGNED_SIZE).map(Slow::new).collect::<Vec<_>>();
    b.iter(|| partition_point_logic4(&ary, |&x| x < Slow::new(0)));
}
