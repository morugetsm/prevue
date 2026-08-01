use std::hint::black_box;
use std::time::{Duration, Instant};

use ahash::RandomState;
use criterion::{
    BenchmarkGroup, Criterion, criterion_group, criterion_main, measurement::WallTime,
};
use serde::Serialize;

// Workloads are named after the template they exercise. Several form pairs whose
// difference isolates one cost: `static` vs `static_bigdata` is data conversion,
// `table` vs `table_text` is interpolation, and an id present in both groups is
// the engine setup a reused renderer amortizes.

const TABLE_SIZE: usize = 100;

/// No directives and no mustaches: parse and serialize with nothing else.
const STATIC_TEMPLATE: &str = r#"<html><head><title>Static</title><meta charset="utf-8"><link rel="stylesheet" href="/app.css"></head><body><header class="site-header"><nav><ul class="nav-list"><li><a href="/">Home</a></li><li><a href="/docs">Docs</a></li><li><a href="/blog">Blog</a></li><li><a href="/about">About</a></li></ul></nav></header><main id="content"><section class="hero"><h1>Static document</h1><p>A representative page with no template syntax at all.</p></section><section class="features"><article class="card"><h2>First</h2><p>Some prose that stands in for real body copy.</p></article><article class="card"><h2>Second</h2><p>Some prose that stands in for real body copy.</p></article><article class="card"><h2>Third</h2><p>Some prose that stands in for real body copy.</p></article></section><table class="data"><thead><tr><th>Name</th><th>Value</th></tr></thead><tbody><tr><td>alpha</td><td>1</td></tr><tr><td>beta</td><td>2</td></tr><tr><td>gamma</td><td>3</td></tr></tbody></table></main><footer class="site-footer"><p>&copy; 2026</p></footer></body></html>"#;

/// Text interpolation alone.
const TEXT_TEMPLATE: &str =
    r#"<article><h1>{{ title }}</h1><p>{{ subtitle }}</p><footer>{{ year }}</footer></article>"#;

/// Attribute binding alone.
const BIND_TEMPLATE: &str = r#"<div :class="{ active: active, muted: !active }">state</div>"#;

/// `v-for` alone: a literal body, so no expression is evaluated per item.
const LIST_TEMPLATE: &str = r#"<ul><li v-for="item in list">entry</li></ul>"#;

/// A realistic page: a loop, a bound attribute and several interpolations.
const PAGE_TEMPLATE: &str = r#"<html><head><title>Members {{ year }}</title></head><body><h1>Members {{ year }}</h1><ul><li v-for="item, index in list" :class="{ adult: item.age >= 18 }">{{ index }}: <b>{{ item.name }}</b> ({{ item.age }})</li></ul></body></html>"#;

/// Nested `v-for` at scale with literal cells.
const TABLE_TEMPLATE: &str = r#"<table><tbody><tr v-for="row in table"><td v-for="col in row">cell</td></tr></tbody></table>"#;

/// `TABLE_TEMPLATE` with an interpolated cell, so the gap is 10,000 evaluations.
const TABLE_TEXT_TEMPLATE: &str = r#"<table><tbody><tr v-for="row in table"><td v-for="col in row">{{ col }}</td></tr></tbody></table>"#;

/// `render` rebuilds the JavaScript engine per call, which dominates anything
/// small. Two workloads are enough to characterize it; per-template detail
/// belongs in `renderer`, where the numbers reflect the template.
fn render(c: &mut Criterion) {
    let mut group = c.benchmark_group("render");
    bench(&mut group, "static", STATIC_TEMPLATE, &text_input());
    bench(&mut group, "page", PAGE_TEMPLATE, &page_input());
    group.finish();
}

fn renderer(c: &mut Criterion) {
    let mut group = c.benchmark_group("renderer");
    bench_reused(&mut group, "static", STATIC_TEMPLATE, &text_input());
    bench_reused(
        &mut group,
        "static_bigdata",
        STATIC_TEMPLATE,
        &table_input(),
    );
    bench_reused(&mut group, "text", TEXT_TEMPLATE, &text_input());
    bench_reused(&mut group, "bind", BIND_TEMPLATE, &bind_input());
    bench_reused(&mut group, "list", LIST_TEMPLATE, &list_input());
    bench_reused(&mut group, "page", PAGE_TEMPLATE, &page_input());
    bench_reused(&mut group, "table", TABLE_TEMPLATE, &table_input());
    bench_reused(
        &mut group,
        "table_text",
        TABLE_TEXT_TEMPLATE,
        &table_input(),
    );
    group.finish();
}

type Group<'a> = BenchmarkGroup<'a, WallTime>;

fn bench<T: Serialize>(group: &mut Group<'_>, id: &str, template: &str, input: &T) {
    bench_renders(group, id, || {
        prevue::render(black_box(template), black_box(input)).unwrap()
    });
}

fn bench_reused<T: Serialize>(group: &mut Group<'_>, id: &str, template: &str, input: &T) {
    let mut renderer = prevue::Renderer::new().unwrap();
    bench_renders(group, id, move || {
        renderer
            .render(black_box(template), black_box(input))
            .unwrap()
    });
}

/// Time `render` alone, then check its output outside the timed region. A reused
/// renderer carries state between calls, so millions of iterations here are a
/// stronger stability check than any test.
fn bench_renders<F: FnMut() -> String>(group: &mut Group<'_>, id: &str, mut render: F) {
    let expected = output_hash(&render());

    group.bench_function(id, |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;

            for _ in 0..iters {
                let start = Instant::now();
                let output = black_box(render());
                total += start.elapsed();

                let hash = output_hash(&output);
                assert_eq!(
                    hash, expected,
                    "output changed: {expected:#018x} != {hash:#018x}"
                );
            }

            total
        });
    });
}

fn output_hash(output: &str) -> u64 {
    // Fixed seeds so a hash is comparable across runs.
    const SEEDS: [u64; 4] = [
        0x243f_6a88_85a3_08d3,
        0x1319_8a2e_0370_7344,
        0xa409_3822_299f_31d0,
        0x082e_fa98_ec4e_6c89,
    ];

    RandomState::with_seeds(SEEDS[0], SEEDS[1], SEEDS[2], SEEDS[3]).hash_one(output)
}

#[derive(Serialize)]
struct Text {
    title: &'static str,
    subtitle: &'static str,
    year: u16,
}

fn text_input() -> Text {
    Text {
        title: "Hello",
        subtitle: "Template benchmark",
        year: 2026,
    }
}

#[derive(Serialize)]
struct Bind {
    active: bool,
}

fn bind_input() -> Bind {
    Bind { active: true }
}

#[derive(Serialize)]
struct List {
    list: Vec<&'static str>,
}

fn list_input() -> List {
    List {
        list: vec!["one", "two", "three", "four", "five"],
    }
}

#[derive(Serialize)]
struct Page {
    year: u16,
    list: Vec<Entry>,
}

#[derive(Serialize)]
struct Entry {
    name: &'static str,
    age: u8,
}

fn page_input() -> Page {
    Page {
        year: 2026,
        list: vec![
            Entry {
                name: "James",
                age: 16,
            },
            Entry {
                name: "John",
                age: 27,
            },
            Entry {
                name: "Alice",
                age: 22,
            },
            Entry {
                name: "Annie",
                age: 17,
            },
        ],
    }
}

#[derive(Serialize)]
struct Table {
    table: Vec<Vec<usize>>,
}

fn table_input() -> Table {
    let table = (0..TABLE_SIZE)
        .map(|_| (0..TABLE_SIZE).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    Table { table }
}

criterion_main!(benches);
criterion_group! {
    name = benches;
    config = new_criterion();
    targets = render, renderer
}

fn new_criterion() -> Criterion {
    Criterion::default()
        .sample_size(500)
        .confidence_level(0.98)
        .significance_level(0.02)
        .warm_up_time(Duration::from_secs(5))
        .measurement_time(Duration::from_secs(60))
}
