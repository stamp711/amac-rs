#![feature(generators, generator_trait)]

use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
    task::Poll,
};

use amac::{async_load, prefetch, LocalPool, PollOnce};
use clf::cache_line_flush_with_slice;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use futures::Future;

fn gen_vec(len: usize) -> Vec<i32> {
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(rand::random());
    }
    vec.sort_unstable();
    vec
}

fn binary_search(v: &[i32], targets: &[i32]) {
    for t in targets {
        let res = binary_search_inner(v, *t);
        let _ = black_box(res);
    }
}

fn binary_search_inner(v: &[i32], target: i32) {
    let mut left = 0;
    let mut right = v.len();

    while left < right {
        let mid = (left + right) / 2;
        let mid_ref = unsafe { v.get_unchecked(mid) };

        if *mid_ref < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    let res = if left == v.len() {
        None
    } else {
        Some(unsafe { v.get_unchecked(left) })
    };

    let _ = black_box(res);
}

fn binary_search_amac(v: &[i32], targets: &[i32]) {
    let futs = targets
        .iter()
        .map(|target| Box::pin(binary_search_amac_inner(v, *target)))
        .collect::<Vec<_>>();

    let exe = LocalPool::from_futures(futs);
    exe.run_until_finish();
}

fn binary_search_amac_manual_execute(v: &[i32], targets: &[i32]) {
    let mut futures = targets
        .iter()
        .map(|target| binary_search_amac_inner(v, *target))
        .collect::<Vec<_>>();
    let mut futs = Vec::with_capacity(futures.len());
    for fut in &mut futures {
        let pf = unsafe { Pin::new_unchecked(fut) };
        futs.push(pf);
    }

    let cx = &mut futures::task::Context::from_waker(futures::task::noop_waker_ref());
    while !futs.is_empty() {
        for i in 0..futs.len() {
            loop {
                let mut bs = match futs.get_mut(i) {
                    Some(bs) => bs,
                    None => break,
                };

                match Pin::new(&mut bs).poll(cx) {
                    Poll::Pending => break,
                    Poll::Ready(_) => {
                        futs.swap_remove(i);
                    }
                }
            }
        }
    }
}

async fn binary_search_amac_inner(v: &[i32], target: i32) {
    let mut left = 0;
    let mut right = v.len();

    while left < right {
        let mid = (left + right) / 2;
        let mid_ref = unsafe { v.get_unchecked(mid) };

        prefetch(mid_ref);
        PollOnce::new().await;

        if *mid_ref < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    let res = if left == v.len() {
        None
    } else {
        let r = unsafe { v.get_unchecked(left) };
        prefetch(r);
        PollOnce::new().await;
        Some(r)
    };

    let _ = black_box(res);
}

fn binary_search_amac_generator_manual_execute(v: &[i32], targets: &[i32]) {
    let mut gens = targets
        .iter()
        .map(|target| binary_search_amac_inner_generator(v, *target))
        .collect::<Vec<_>>();
    let mut bss = Vec::with_capacity(gens.len());
    for gen in &mut gens {
        let bs = Pin::new(gen);
        bss.push(bs);
    }

    while !bss.is_empty() {
        for i in 0..bss.len() {
            loop {
                let mut bs = match bss.get_mut(i) {
                    Some(bs) => bs,
                    None => break,
                };

                match Pin::new(&mut bs).resume(()) {
                    GeneratorState::Yielded(_) => break,
                    GeneratorState::Complete(_) => {
                        bss.swap_remove(i);
                    }
                }
            }
        }
    }
}

fn binary_search_amac_inner_generator(
    v: &[i32],
    target: i32,
) -> impl Generator<Yield = (), Return = ()> + '_ {
    move || {
        let mut left = 0;
        let mut right = v.len();

        while left < right {
            let mid = (left + right) / 2;
            let mid_ref = unsafe { v.get_unchecked(mid) };

            prefetch(mid_ref);
            yield;

            if *mid_ref < target {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        let res = if left == v.len() {
            None
        } else {
            let r = unsafe { v.get_unchecked(left) };
            prefetch(r);
            yield;
            Some(r)
        };

        let _ = black_box(res);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let vec = gen_vec(256 * 1024 * 1024);
    let targets = gen_vec(300);

    let mut group = c.benchmark_group("binary_search");

    let setup = || {
        cache_line_flush_with_slice(&vec);
    };

    group.bench_function("normal", |b| {
        b.iter_with_setup(setup, |_| {
            binary_search(black_box(&vec), black_box(&targets))
        })
    });

    group.bench_function("amac async boxed", |b| {
        b.iter_with_setup(setup, |_| {
            binary_search_amac(black_box(&vec), black_box(&targets))
        })
    });

    group.bench_function("amac async unboxed", |b| {
        b.iter_with_setup(setup, |_| {
            binary_search_amac_manual_execute(black_box(&vec), black_box(&targets))
        })
    });

    group.bench_function("amac generator unboxed", |b| {
        b.iter_with_setup(setup, |_| {
            binary_search_amac_generator_manual_execute(black_box(&vec), black_box(&targets))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
