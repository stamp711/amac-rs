use std::cell::Cell;

use amac::{AsyncPrefetch, LocalPool};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

type ABox<T> = AsyncPrefetch<Box<T>>;

type VTest = Vec<ABox<ABox<ABox<ABox<ABox<i32>>>>>>;

fn make_pointer_vec<T>(len: usize) -> VTest {
    let mut res = vec![];
    for _ in 0..len {
        res.push(AsyncPrefetch::new(Box::new(AsyncPrefetch::new(Box::new(
            AsyncPrefetch::new(Box::new(AsyncPrefetch::new(Box::new(AsyncPrefetch::new(
                Box::new(rand::random()),
            ))))),
        )))));
    }
    res
}

fn add_pointer_vec(vec: VTest) -> i32 {
    let mut res = 0;
    for i in vec {
        res += i.load().load().load().load().load();
    }
    res
}

fn add_pointer_vec_with_prefetch(vec: VTest) -> i32 {
    let res = Cell::new(0);

    let mut exe = LocalPool::new();
    let step = 100;
    for i in 0..step {
        let fut = Box::pin(add_to_res(&vec, &res, i, step));
        exe.add_future(fut);
    }
    exe.run_until_finish();

    res.get()
}

async fn add_to_res(vec: &VTest, res: &Cell<i32>, start: usize, step: usize) {
    let mut sub_res = 0;

    let mut id = start;
    while id < vec.len() {
        sub_res += vec[id]
            .prefetch_load()
            .await
            .prefetch_load()
            .await
            .prefetch_load()
            .await
            .prefetch_load()
            .await
            .prefetch_load()
            .await;
        id += step;
    }

    res.set(res.get() + sub_res);
}

fn flush_vtest(vec: &VTest) {
    for i in vec {
        let ptr1 = i.load() as *const _ as *const u8;
        let ptr2 = i.load().load() as *const _ as *const u8;
        let ptr3 = i.load().load().load() as *const _ as *const u8;
        let ptr4 = i.load().load().load().load() as *const _ as *const u8;
        let ptr5 = i.load().load().load().load().load() as *const _ as *const u8;

        clf::cache_line_flush_with_ptr(ptr1, unsafe { ptr1.offset(8) });
        clf::cache_line_flush_with_ptr(ptr2, unsafe { ptr2.offset(8) });
        clf::cache_line_flush_with_ptr(ptr3, unsafe { ptr3.offset(8) });
        clf::cache_line_flush_with_ptr(ptr4, unsafe { ptr4.offset(8) });
        clf::cache_line_flush_with_ptr(ptr5, unsafe { ptr5.offset(8) });
    }
    clf::cache_line_flush_with_slice(vec);
}

fn criterion_benchmark(c: &mut Criterion) {
    let vec = make_pointer_vec::<i32>(1000);

    {
        let res1 = add_pointer_vec(vec.clone());
        let res2 = add_pointer_vec_with_prefetch(vec.clone());
        assert_eq!(res1, res2);
    }

    let mut group = c.benchmark_group("add_pointer_vec");

    let setup = || {
        let v = vec.clone();
        flush_vtest(&v);
        v
    };

    group.bench_function("prefetch", |b| {
        b.iter_with_setup(setup, |vec| add_pointer_vec_with_prefetch(black_box(vec)))
    });

    group.bench_function("no_prefetch", |b| {
        b.iter_with_setup(setup, |vec| add_pointer_vec(black_box(vec)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
