use amac::{async_load, LocalPool};
use clf::cache_line_flush_with_slice;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

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
    let mut futs = vec![];
    for t in targets {
        let fut = binary_search_amac_inner(v, *t);
        futs.push(Box::pin(fut));
    }

    let exe = LocalPool::from_futures(futs);
    exe.run_until_finish();
}

async fn binary_search_amac_inner(v: &[i32], target: i32) {
    let mut left = 0;
    let mut right = v.len();

    while left < right {
        let mid = (left + right) / 2;
        let mid_ref = unsafe { v.get_unchecked(mid) };

        if *async_load(mid_ref).await < target {
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

fn criterion_benchmark(c: &mut Criterion) {
    let vec = gen_vec(1024 * 1024 * 1024);
    let targets = (0..1000).map(|_| rand::random()).collect::<Vec<_>>();

    let mut group = c.benchmark_group("binary_search");

    let setup = || {
        cache_line_flush_with_slice(&vec);
    };

    group.bench_function("normal", |b| {
        b.iter_with_setup(setup, |_| {
            binary_search(black_box(&vec), black_box(&targets))
        })
    });

    group.bench_function("amac", |b| {
        b.iter_with_setup(setup, |_| {
            binary_search_amac(black_box(&vec), black_box(&targets))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
