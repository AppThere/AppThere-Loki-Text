//! Parsing benchmarks.
//!
//! Measures `parse_document` throughput across document sizes and content types.
//! Performance targets:
//!   * 10 000 paragraphs: <500 ms
//!   * 10 000 formatted paragraphs: <600 ms
//!   * 100×20 table: <300 ms

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use odt_format::parser::parse_document;

mod generators;

// ── Parse scaling: 100, 1K, 10K, 50K plain paragraphs ────────────────────────

fn bench_parse_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_scaling");
    for &size in &[100_usize, 1_000, 10_000, 50_000] {
        let xml = generators::paragraphs_xml(size);
        group.throughput(Throughput::Bytes(xml.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_paragraphs")),
            &xml,
            |b, xml| b.iter(|| parse_document(black_box(xml)).unwrap()),
        );
    }
    group.finish();
}

// ── Formatted text: bold + italic spans ──────────────────────────────────────

fn bench_parse_formatted(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_formatted");
    for &size in &[100_usize, 1_000, 10_000] {
        let xml = generators::formatted_xml(size);
        group.throughput(Throughput::Bytes(xml.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_paragraphs")),
            &xml,
            |b, xml| b.iter(|| parse_document(black_box(xml)).unwrap()),
        );
    }
    group.finish();
}

// ── Tables ────────────────────────────────────────────────────────────────────

fn bench_parse_tables(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_tables");
    for &(rows, cols) in &[(10_usize, 5_usize), (50, 10), (100, 20)] {
        let xml = generators::table_xml(rows, cols);
        group.throughput(Throughput::Elements((rows * cols) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{rows}x{cols}_table")),
            &xml,
            |b, xml| b.iter(|| parse_document(black_box(xml)).unwrap()),
        );
    }
    group.finish();
}

// ── Flat bullet lists ─────────────────────────────────────────────────────────

fn bench_parse_lists(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_lists");
    for &items in &[50_usize, 500, 5_000] {
        let xml = generators::list_xml(items);
        group.throughput(Throughput::Elements(items as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{items}_items")),
            &xml,
            |b, xml| b.iter(|| parse_document(black_box(xml)).unwrap()),
        );
    }
    group.finish();
}

// ── Many named styles ─────────────────────────────────────────────────────────

fn bench_parse_many_styles(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_many_styles");
    for &n_styles in &[10_usize, 100, 1_000] {
        let xml = generators::many_styles_xml(1_000, n_styles);
        group.throughput(Throughput::Elements(n_styles as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{n_styles}_styles")),
            &xml,
            |b, xml| b.iter(|| parse_document(black_box(xml)).unwrap()),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_parse_scaling,
    bench_parse_formatted,
    bench_parse_tables,
    bench_parse_lists,
    bench_parse_many_styles,
);
criterion_main!(benches);
