//! Full-pipeline benchmark: parse → to_lexical → from_lexical → write.
//!
//! This is the end-to-end benchmark exercising every layer of the stack.
//! Performance target: 10 000-paragraph document in <1 000 ms.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use odt_format::{
    lexical::{from_lexical, to_lexical},
    parser::parse_document,
};

mod generators;

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    for &size in &[100_usize, 1_000, 10_000] {
        let xml = generators::paragraphs_xml(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_paragraphs")),
            &xml,
            |b, xml| {
                b.iter(|| {
                    // 1. Parse ODT XML
                    let doc = parse_document(black_box(xml)).unwrap();
                    // 2. Convert to Lexical (frontend format)
                    let lex = to_lexical(&doc);
                    // 3. Convert back from Lexical (simulates a save)
                    let doc2 = from_lexical(lex, doc.styles.clone(), doc.metadata.clone());
                    // 4. Write content XML
                    let _ = doc2.to_content_xml().unwrap();
                })
            },
        );
    }
    group.finish();
}

fn bench_full_pipeline_formatted(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline_formatted");

    for &size in &[100_usize, 1_000, 10_000] {
        let xml = generators::formatted_xml(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_paragraphs")),
            &xml,
            |b, xml| {
                b.iter(|| {
                    let doc = parse_document(black_box(xml)).unwrap();
                    let lex = to_lexical(&doc);
                    let doc2 = from_lexical(lex, doc.styles.clone(), doc.metadata.clone());
                    let _ = doc2.to_content_xml().unwrap();
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_full_pipeline, bench_full_pipeline_formatted);
criterion_main!(benches);
