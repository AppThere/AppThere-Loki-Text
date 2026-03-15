//! Lexical ↔ ODT conversion benchmarks.
//!
//! Measures `to_lexical` and `from_lexical` throughput at various scales.
//! Performance targets:
//!   * `to_lexical`   10 000 blocks: <50 ms
//!   * `from_lexical` 10 000 blocks: <50 ms
//!   * Full round-trip 10 000 blocks: <100 ms

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use odt_format::lexical::{from_lexical, to_lexical};

mod generators;

// ── ODT → Lexical ─────────────────────────────────────────────────────────────

fn bench_to_lexical(c: &mut Criterion) {
    let mut group = c.benchmark_group("to_lexical");
    for &size in &[100_usize, 1_000, 10_000] {
        let doc = generators::simple_document(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_blocks")),
            &doc,
            |b, doc| b.iter(|| to_lexical(black_box(doc))),
        );
    }
    group.finish();
}

// ── Lexical → ODT ─────────────────────────────────────────────────────────────

fn bench_from_lexical(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_lexical");
    for &size in &[100_usize, 1_000, 10_000] {
        let doc = generators::simple_document(size);
        let lex = to_lexical(&doc);
        let styles = doc.styles.clone();
        let meta = doc.metadata.clone();
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_blocks")),
            &(lex, styles, meta),
            |b, (lex, styles, meta)| {
                b.iter(|| {
                    from_lexical(
                        black_box(lex.clone()),
                        black_box(styles.clone()),
                        black_box(meta.clone()),
                    )
                })
            },
        );
    }
    group.finish();
}

// ── Lexical round-trip (to → from) ───────────────────────────────────────────

fn bench_lexical_round_trip(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexical_round_trip");
    for &size in &[100_usize, 1_000, 10_000] {
        let doc = generators::simple_document(size);
        let styles = doc.styles.clone();
        let meta = doc.metadata.clone();
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_blocks")),
            &(doc, styles, meta),
            |b, (doc, styles, meta)| {
                b.iter(|| {
                    let lex = to_lexical(black_box(doc));
                    from_lexical(lex, styles.clone(), meta.clone())
                })
            },
        );
    }
    group.finish();
}

// ── Formatted-text conversion ─────────────────────────────────────────────────

fn bench_to_lexical_formatted(c: &mut Criterion) {
    let mut group = c.benchmark_group("to_lexical_formatted");
    for &size in &[100_usize, 1_000, 10_000] {
        let doc = generators::formatted_document(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_blocks")),
            &doc,
            |b, doc| b.iter(|| to_lexical(black_box(doc))),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_to_lexical,
    bench_from_lexical,
    bench_lexical_round_trip,
    bench_to_lexical_formatted,
);
criterion_main!(benches);
