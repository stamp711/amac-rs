// #![feature(allocator_api)]

// use std::{alloc::Allocator, cell::Cell, collections::HashSet, mem::size_of};

// use amac::{AsyncPrefetch, LocalPool};
// use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use futures::Future;

// type ABox<T> = AsyncPrefetch<Box<T>>;

// type VTest = Vec<ABox<i32>>;

// fn make_pointer_vec<T>(len: usize) -> VTest {
//     let mut h = HashSet::new();
//     for _ in 0..len {
//         h.insert(AsyncPrefetch::new(Box::new(rand::random())));
//     }
//     let mut res = vec![];
//     for i in h {
//         res.push(i);
//     }
//     res
// }

// fn add_pointer_vec(vec: &VTest) -> i32 {
//     let mut res = 0;
//     for i in vec {
//         res += i.load();
//     }
//     res
// }

// fn add_pointer_vec_with_prefetch(vec: &VTest) -> i32 {
//     let res = Cell::new(0);

//     let step = 500;
//     let mut futs = vec![];
//     for i in 0..step {
//         let fut = add_to_res(&vec, &res, i, step);
//         futs.push(Box::pin(fut));
//     }

//     let exe = LocalPool::from_futures(futs);
//     exe.run_until_finish();

//     res.get()
// }

// fn add_to_res<'a, 'b>(
//     vec: &'a VTest,
//     res: &'a Cell<i32>,
//     start: usize,
//     step: usize,
// ) -> impl Future<Output = ()> + 'a {
//     async move {
//         let mut sub_res = 0;

//         let mut id = start;
//         while id < vec.len() {
//             sub_res += unsafe { vec.get_unchecked(id).prefetch_load() }.await;
//             id += step;
//         }

//         // res.set(res.get() + sub_res);
//     }
// }

// fn flush_vtest(vec: &VTest) {
//     for i in vec {
//         let ptr1 = i.load() as *const _ as *const u8;
//         clf::cache_line_flush_with_ptr(ptr1, unsafe { ptr1.offset(8) });
//     }
//     clf::cache_line_flush_with_slice(vec);
// }

// fn criterion_benchmark(c: &mut Criterion) {
//     let vec = make_pointer_vec::<i32>(10 * 1024 * 1024);

//     let mut group = c.benchmark_group("add_pointer_vec");
//     group.sample_size(10);

//     let setup = || {
//         let v = vec.clone();
//         // flush_vtest(&v);
//         v
//     };

//     group.bench_function("no_prefetch", |b| {
//         b.iter(|| {
//             let res = add_pointer_vec(black_box(&vec));
//             black_box(res)
//         })
//     });

//     group.bench_function("prefetch", |b| {
//         b.iter(|| {
//             let res = add_pointer_vec_with_prefetch(black_box(&vec));
//             black_box(res)
//         })
//     });
// }

// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);

fn main() {}
