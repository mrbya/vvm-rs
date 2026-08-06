//! Criterion benchmarks for packed-value operations.

mod packed_support;

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use packed_support::{
    REPRESENTATIVE_WIDTHS, configure_group, scalar_value, throughput_elements, words_for_width,
};
use vvm_core::{
    Bits, SignedBits, extract_packed, extract_signed, extract_unsigned, insert_packed,
    insert_signed, insert_unsigned,
};

/// Benchmarks scalar extraction and insertion hot paths.
fn scalar_packed_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("packed/scalar");
    configure_group(&mut group);

    for width in REPRESENTATIVE_WIDTHS
        .into_iter()
        .filter(|width| *width <= 64)
    {
        let words = words_for_width(width);
        let value = scalar_value(width);

        throughput_elements(&mut group, u64::try_from(width).unwrap_or_default());

        group.bench_with_input(
            BenchmarkId::new("extract", width),
            &width,
            |bench, &bench_width| {
                bench.iter(|| {
                    let extracted =
                        extract_unsigned(black_box(&words), bench_width, 0, bench_width);
                    black_box(extracted)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("insert", width),
            &width,
            |bench, &bench_width| {
                bench.iter(|| {
                    let mut output = vec![0_u32; words.len()];
                    let inserted = insert_unsigned(&mut output, bench_width, 0, bench_width, value);
                    black_box(inserted)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("extract-signed", width),
            &width,
            |bench, &bench_width| {
                bench.iter(|| {
                    let extracted = extract_signed(black_box(&words), bench_width, 0, bench_width);
                    black_box(extracted)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("insert-signed", width),
            &width,
            |bench, &bench_width| {
                bench.iter(|| {
                    let mut output = vec![0_u32; words.len()];
                    let inserted = insert_signed(&mut output, bench_width, 0, bench_width, -5);
                    black_box(inserted)
                });
            },
        );
    }

    group.finish();
}

/// Benchmarks deterministic `Bits` and `SignedBits` operations.
fn typed_packed_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("packed/typed");
    configure_group(&mut group);

    let bits_65 = Bits::<65>::from_words_le(words_for_width(65)).unwrap_or_default();
    let bits_256 = Bits::<256>::from_words_le(words_for_width(256)).unwrap_or_default();
    let bits_1024 = Bits::<1024>::from_words_le(words_for_width(1024)).unwrap_or_default();
    let signed_65 = SignedBits::<65>::from_words_le(words_for_width(65)).unwrap_or_default();
    let signed_128 = SignedBits::<128>::from_words_le(words_for_width(128)).unwrap_or_default();

    group.bench_function("bits/construct/65", |bench| {
        let words = words_for_width(65);
        bench.iter(|| black_box(Bits::<65>::from_words_le(black_box(&words))));
    });

    group.bench_function("bits/equality/256", |bench| {
        bench.iter(|| black_box(black_box(&bits_256) == black_box(&bits_256)));
    });

    group.bench_function("bits/bit/1024", |bench| {
        bench.iter(|| black_box(bits_1024.bit(black_box(511))));
    });

    group.bench_function("signed/construct/65", |bench| {
        let words = words_for_width(65);
        bench.iter(|| black_box(SignedBits::<65>::from_words_le(black_box(&words))));
    });

    group.bench_function("signed/is-negative/128", |bench| {
        bench.iter(|| black_box(signed_128.is_negative()));
    });

    group.bench_function("range/extract/65", |bench| {
        bench.iter(|| {
            black_box(extract_packed::<Bits<65>>(
                black_box(bits_1024.words_le()),
                1024,
                17,
            ))
        });
    });

    group.bench_function("range/insert/65", |bench| {
        let value = bits_65.clone();
        bench.iter(|| {
            let mut output = vec![0_u32; 32];
            black_box(insert_packed(&mut output, 1024, 17, black_box(&value)))
        });
    });

    group.bench_function("range/extract-signed/65", |bench| {
        bench.iter(|| {
            black_box(extract_packed::<SignedBits<65>>(
                black_box(bits_1024.words_le()),
                1024,
                33,
            ))
        });
    });

    group.bench_function("range/insert-signed/65", |bench| {
        let value = signed_65.clone();
        bench.iter(|| {
            let mut output = vec![0_u32; 32];
            black_box(insert_packed(&mut output, 1024, 33, black_box(&value)))
        });
    });

    let _ = bits_65;
    group.finish();
}

criterion_group!(packed_benches, scalar_packed_benches, typed_packed_benches);
criterion_main!(packed_benches);
