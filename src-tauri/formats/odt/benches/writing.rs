//! Writing (serialization) benchmarks.
//!
//! Measures `to_content_xml`, `styles_to_xml`, and `to_meta_xml` throughput.
//! Performance targets:
//!   * Write 10 000 paragraphs: <300 ms
//!   * Full ODT (content + styles + meta) for 10K paragraphs: <400 ms

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

mod generators;

// ── Plain-paragraph write scaling ─────────────────────────────────────────────

fn bench_write_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_scaling");
    for &size in &[100_usize, 1_000, 10_000, 50_000] {
        let doc = generators::simple_document(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_paragraphs")),
            &doc,
            |b, doc| b.iter(|| black_box(doc).to_content_xml().unwrap()),
        );
    }
    group.finish();
}

// ── Formatted text write ──────────────────────────────────────────────────────

fn bench_write_formatted(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_formatted");
    for &size in &[100_usize, 1_000, 10_000] {
        let doc = generators::formatted_document(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_paragraphs")),
            &doc,
            |b, doc| b.iter(|| black_box(doc).to_content_xml().unwrap()),
        );
    }
    group.finish();
}

// ── Style registry serialization ──────────────────────────────────────────────

fn bench_write_styles(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_styles");
    for &n_styles in &[10_usize, 100, 1_000] {
        let doc = generators::styled_document(1_000, n_styles);
        group.throughput(Throughput::Elements(n_styles as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{n_styles}_styles")),
            &doc,
            |b, doc| b.iter(|| black_box(doc).styles_to_xml().unwrap()),
        );
    }
    group.finish();
}

// ── Full ODT output (content.xml + styles.xml + meta.xml) ────────────────────

fn bench_write_full_odt(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_full_odt");
    for &size in &[100_usize, 1_000, 10_000] {
        let doc = generators::simple_document(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_paragraphs")),
            &doc,
            |b, doc| {
                b.iter(|| {
                    let doc = black_box(doc);
                    let _ = doc.to_content_xml().unwrap();
                    let _ = doc.styles_to_xml().unwrap();
                    let _ = doc.to_meta_xml().unwrap();
                })
            },
        );
    }
    group.finish();
}

// ── FODT (flat XML) write ─────────────────────────────────────────────────────

fn bench_write_fodt(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_fodt");
    for &size in &[100_usize, 1_000, 10_000] {
        let doc = generators::simple_document(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_paragraphs")),
            &doc,
            |b, doc| b.iter(|| black_box(doc).to_xml().unwrap()),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_write_scaling,
    bench_write_formatted,
    bench_write_styles,
    bench_write_full_odt,
    bench_write_fodt,
);
criterion_main!(benches);
