use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use eyepiece::{Builder, Field, FieldBuilder, Gmt, PixelScale, SeeingBuilder, SeeingLimited};
use skyangle::SkyAngle;

fn diffraction() -> Field<Gmt> {
    let gmt = Gmt::new();
    FieldBuilder::new(gmt)
        .pixel_scale(PixelScale::NyquistFraction(1))
        .field_of_view(21)
        .build()
}

fn seeing() -> Field<Gmt, SeeingLimited> {
    let gmt = Gmt::new();
    FieldBuilder::new(gmt)
        .pixel_scale(SkyAngle::Arcsecond(0.01))
        .field_of_view(200)
        .seeing_limited(SeeingBuilder::new(16e-2).zenith_angle(SkyAngle::Degree(30.)))
        .build()
}

fn diffraction_limited(c: &mut Criterion) {
    let mut field = diffraction();
    let mut group = c.benchmark_group("eyepiece");
    group.sample_size(25);
    group.measurement_time(Duration::from_secs(60));
    group.bench_function("GMT diffraction limited", |b| {
        b.iter(|| field.intensity(None))
    });
    group.finish();
}

fn seeing_limited(c: &mut Criterion) {
    let mut field = seeing();
    let mut group = c.benchmark_group("eyepiece");
    group.sample_size(25);
    group.bench_function("GMT seeing limited", |b| b.iter(|| field.intensity(None)));
    group.finish();
}
criterion_group!(benches, diffraction_limited, seeing_limited);
criterion_main!(benches);
