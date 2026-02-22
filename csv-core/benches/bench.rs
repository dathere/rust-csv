use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use csv_core::{Reader, ReaderBuilder};

static NFL: &str = include_str!("../../examples/data/bench/nfl.csv");
static GAME: &str = include_str!("../../examples/data/bench/game.csv");
static POP: &str =
    include_str!("../../examples/data/bench/worldcitiespop.csv");
static MBTA: &str =
    include_str!("../../examples/data/bench/gtfs-mbta-stop-times.csv");

macro_rules! bench_core {
    (
        $fn_name:ident, $group:expr, $data:ident,
        fields: $fields:expr, records: $records:expr
    ) => {
        fn $fn_name(c: &mut Criterion) {
            let data = $data.as_bytes();
            let mut g = c.benchmark_group($group);
            g.throughput(Throughput::Bytes(data.len() as u64));

            g.bench_function("field_dfa", |b| {
                let mut rdr = ReaderBuilder::new().build();
                b.iter(|| {
                    rdr.reset();
                    assert_eq!(count_fields(&mut rdr, data), $fields);
                })
            });

            g.bench_function("field_nfa", |b| {
                let mut rdr = ReaderBuilder::new().nfa(true).build();
                b.iter(|| {
                    rdr.reset();
                    assert_eq!(count_fields(&mut rdr, data), $fields);
                })
            });

            g.bench_function("record_dfa", |b| {
                let mut rdr = ReaderBuilder::new().build();
                b.iter(|| {
                    rdr.reset();
                    assert_eq!(count_records(&mut rdr, data), $records);
                })
            });

            g.bench_function("record_nfa", |b| {
                let mut rdr = ReaderBuilder::new().nfa(true).build();
                b.iter(|| {
                    rdr.reset();
                    assert_eq!(count_records(&mut rdr, data), $records);
                })
            });

            g.finish();
        }
    };
}

bench_core!(bench_nfl, "nfl", NFL,
    fields: 130000, records: 10000);
bench_core!(bench_game, "game", GAME,
    fields: 600000, records: 100000);
bench_core!(bench_pop, "pop", POP,
    fields: 140007, records: 20001);
bench_core!(bench_mbta, "mbta", MBTA,
    fields: 90000, records: 10000);

fn count_fields(rdr: &mut Reader, mut data: &[u8]) -> u64 {
    use csv_core::ReadFieldResult::*;

    let mut count = 0;
    let mut field = [0u8; 1024];
    loop {
        let (res, nin, _) = rdr.read_field(data, &mut field);
        data = &data[nin..];
        match res {
            InputEmpty => {}
            OutputFull => panic!("field too large"),
            Field { .. } => {
                count += 1;
            }
            End => break,
        }
    }
    count
}

fn count_records(rdr: &mut Reader, mut data: &[u8]) -> u64 {
    use csv_core::ReadRecordResult::*;

    let mut count = 0;
    let mut record = [0; 8192];
    let mut ends = [0; 32];
    loop {
        let (res, nin, _, _) = rdr.read_record(data, &mut record, &mut ends);
        data = &data[nin..];
        match res {
            InputEmpty => {}
            OutputFull | OutputEndsFull => {
                panic!("field too large")
            }
            Record => count += 1,
            End => break,
        }
    }
    count
}

criterion_group!(benches, bench_nfl, bench_game, bench_pop, bench_mbta,);
criterion_main!(benches);
