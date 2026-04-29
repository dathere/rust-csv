use std::{fmt, io};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use serde::{
    de::{DeserializeOwned, Deserializer, Visitor},
    Deserialize, Serialize,
};

use csv::{
    ByteRecord, Reader, ReaderBuilder, StringRecord, Trim, Writer,
    WriterBuilder,
};

static NFL: &str = include_str!("../examples/data/bench/nfl.csv");
static GAME: &str = include_str!("../examples/data/bench/game.csv");
static POP: &str = include_str!("../examples/data/bench/worldcitiespop.csv");
static MBTA: &str =
    include_str!("../examples/data/bench/gtfs-mbta-stop-times.csv");

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct NFLRowOwned {
    gameid: String,
    qtr: i32,
    min: Option<i32>,
    sec: Option<i32>,
    off: String,
    def: String,
    down: Option<i32>,
    togo: Option<i32>,
    ydline: Option<i32>,
    description: String,
    offscore: i32,
    defscore: i32,
    season: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct NFLRowBorrowed<'a> {
    gameid: &'a str,
    qtr: i32,
    min: Option<i32>,
    sec: Option<i32>,
    off: &'a str,
    def: &'a str,
    down: Option<i32>,
    togo: Option<i32>,
    ydline: Option<i32>,
    description: &'a str,
    offscore: i32,
    defscore: i32,
    season: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GAMERowOwned(String, String, String, String, i32, String);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GAMERowBorrowed<'a>(&'a str, &'a str, &'a str, &'a str, i32, &'a str);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct POPRowOwned {
    country: String,
    city: String,
    accent_city: String,
    region: String,
    population: Option<i32>,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct POPRowBorrowed<'a> {
    country: &'a str,
    city: &'a str,
    accent_city: &'a str,
    region: &'a str,
    population: Option<i32>,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MBTARowOwned {
    trip_id: String,
    arrival_time: String,
    departure_time: String,
    stop_id: String,
    stop_sequence: i32,
    stop_headsign: String,
    pickup_type: i32,
    drop_off_type: i32,
    timepoint: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MBTARowBorrowed<'a> {
    trip_id: &'a str,
    arrival_time: &'a str,
    departure_time: &'a str,
    stop_id: &'a str,
    stop_sequence: i32,
    stop_headsign: &'a str,
    pickup_type: i32,
    drop_off_type: i32,
    timepoint: i32,
}

#[derive(Default)]
struct ByteCounter {
    count: usize,
}
impl io::Write for ByteCounter {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.count += data.len();
        Ok(data.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// --- Helper functions ---

fn count_deserialize_owned_bytes<R, D>(rdr: &mut Reader<R>) -> u64
where
    R: io::Read,
    D: DeserializeOwned,
{
    let mut count = 0;
    let mut rec = ByteRecord::new();
    while rdr.read_byte_record(&mut rec).unwrap() {
        let _: D = rec.deserialize(None).unwrap();
        count += 1;
    }
    count
}

fn count_deserialize_owned_str<R, D>(rdr: &mut Reader<R>) -> u64
where
    R: io::Read,
    D: DeserializeOwned,
{
    let mut count = 0;
    for rec in rdr.deserialize::<D>() {
        let _ = rec.unwrap();
        count += 1;
    }
    count
}

fn count_iter_bytes<R: io::Read>(rdr: &mut Reader<R>) -> u64 {
    let mut count = 0;
    for rec in rdr.byte_records() {
        count += rec.unwrap().len() as u64;
    }
    count
}

fn count_iter_str<R: io::Read>(rdr: &mut Reader<R>) -> u64 {
    let mut count = 0;
    for rec in rdr.records() {
        count += rec.unwrap().len() as u64;
    }
    count
}

fn count_read_bytes<R: io::Read>(rdr: &mut Reader<R>) -> u64 {
    let mut count = 0;
    let mut rec = ByteRecord::new();
    while rdr.read_byte_record(&mut rec).unwrap() {
        count += rec.len() as u64;
    }
    count
}

fn count_read_str<R: io::Read>(rdr: &mut Reader<R>) -> u64 {
    let mut count = 0;
    let mut rec = StringRecord::new();
    while rdr.read_record(&mut rec).unwrap() {
        count += rec.len() as u64;
    }
    count
}

fn collect_records(data: &[u8]) -> Vec<ByteRecord> {
    let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(data);
    rdr.byte_records().collect::<Result<Vec<_>, _>>().unwrap()
}

// --- Benchmark definitions ---

macro_rules! bench_dataset {
    (
        $fn_name:ident, $group:expr, $data:ident,
        field_count: $fc:expr,
        serde_count: $sc:expr,
        serde_headers: $sh:expr,
        owned: $owned:ty,
        borrowed: $borrowed:ty
    ) => {
        fn $fn_name(c: &mut Criterion) {
            let data = $data.as_bytes();
            let mut g = c.benchmark_group($group);
            g.throughput(Throughput::Bytes(data.len() as u64));

            g.bench_function("iter_bytes", |b| {
                b.iter(|| {
                    let mut rdr = ReaderBuilder::new()
                        .has_headers(false)
                        .from_reader(data);
                    assert_eq!(count_iter_bytes(&mut rdr), $fc);
                })
            });

            g.bench_function("iter_str", |b| {
                b.iter(|| {
                    let mut rdr = ReaderBuilder::new()
                        .has_headers(false)
                        .from_reader(data);
                    assert_eq!(count_iter_str(&mut rdr), $fc);
                })
            });

            g.bench_function("read_bytes", |b| {
                b.iter(|| {
                    let mut rdr = ReaderBuilder::new()
                        .has_headers(false)
                        .from_reader(data);
                    assert_eq!(count_read_bytes(&mut rdr), $fc);
                })
            });

            g.bench_function("read_str", |b| {
                b.iter(|| {
                    let mut rdr = ReaderBuilder::new()
                        .has_headers(false)
                        .from_reader(data);
                    assert_eq!(count_read_str(&mut rdr), $fc);
                })
            });

            g.bench_function("deserialize_owned_bytes", |b| {
                b.iter(|| {
                    let mut rdr = ReaderBuilder::new()
                        .has_headers($sh)
                        .from_reader(data);
                    assert_eq!(
                        count_deserialize_owned_bytes::<_, $owned>(&mut rdr),
                        $sc
                    );
                })
            });

            g.bench_function("deserialize_owned_str", |b| {
                b.iter(|| {
                    let mut rdr = ReaderBuilder::new()
                        .has_headers($sh)
                        .from_reader(data);
                    assert_eq!(
                        count_deserialize_owned_str::<_, $owned>(&mut rdr),
                        $sc
                    );
                })
            });

            g.bench_function("deserialize_borrowed_bytes", |b| {
                b.iter(|| {
                    let mut rdr = ReaderBuilder::new()
                        .has_headers($sh)
                        .from_reader(data);
                    let mut count = 0u64;
                    let mut rec = ByteRecord::new();
                    while rdr.read_byte_record(&mut rec).unwrap() {
                        let _: $borrowed = rec.deserialize(None).unwrap();
                        count += 1;
                    }
                    count
                })
            });

            g.bench_function("deserialize_borrowed_str", |b| {
                b.iter(|| {
                    let mut rdr = ReaderBuilder::new()
                        .has_headers($sh)
                        .from_reader(data);
                    let mut count = 0u64;
                    let mut rec = StringRecord::new();
                    while rdr.read_record(&mut rec).unwrap() {
                        let _: $borrowed = rec.deserialize(None).unwrap();
                        count += 1;
                    }
                    count
                })
            });

            let ser_values: Vec<$owned> = ReaderBuilder::new()
                .has_headers($sh)
                .from_reader(data)
                .deserialize()
                .collect::<Result<_, _>>()
                .unwrap();
            g.bench_function("serialize", |b| {
                b.iter(|| {
                    let mut wtr = WriterBuilder::new()
                        .has_headers($sh)
                        .from_writer(ByteCounter::default());
                    for val in &ser_values {
                        wtr.serialize(val).unwrap();
                    }
                })
            });

            g.finish();
        }
    };
}

bench_dataset!(
    bench_nfl, "nfl", NFL,
    field_count: 130000,
    serde_count: 9999,
    serde_headers: true,
    owned: NFLRowOwned,
    borrowed: NFLRowBorrowed
);

bench_dataset!(
    bench_game, "game", GAME,
    field_count: 600000,
    serde_count: 100000,
    serde_headers: false,
    owned: GAMERowOwned,
    borrowed: GAMERowBorrowed
);

bench_dataset!(
    bench_pop, "pop", POP,
    field_count: 140007,
    serde_count: 20000,
    serde_headers: true,
    owned: POPRowOwned,
    borrowed: POPRowBorrowed
);

bench_dataset!(
    bench_mbta, "mbta", MBTA,
    field_count: 90000,
    serde_count: 9999,
    serde_headers: true,
    owned: MBTARowOwned,
    borrowed: MBTARowBorrowed
);

fn bench_nfl_trimmed(c: &mut Criterion) {
    let data = NFL.as_bytes();
    let mut g = c.benchmark_group("nfl_trimmed");
    g.throughput(Throughput::Bytes(data.len() as u64));

    g.bench_function("iter_bytes", |b| {
        b.iter(|| {
            let mut rdr = ReaderBuilder::new()
                .has_headers(false)
                .trim(Trim::All)
                .from_reader(data);
            assert_eq!(count_iter_bytes(&mut rdr), 130000);
        })
    });

    g.bench_function("iter_str", |b| {
        b.iter(|| {
            let mut rdr = ReaderBuilder::new()
                .has_headers(false)
                .trim(Trim::All)
                .from_reader(data);
            assert_eq!(count_iter_str(&mut rdr), 130000);
        })
    });

    g.finish();
}

fn bench_nfl_write(c: &mut Criterion) {
    let data = NFL.as_bytes();
    let records = collect_records(data);
    let mut g = c.benchmark_group("nfl_write");
    g.throughput(Throughput::Bytes(data.len() as u64));

    g.bench_function("record", |b| {
        b.iter(|| {
            let mut wtr = Writer::from_writer(vec![]);
            for r in &records {
                wtr.write_record(r).unwrap();
            }
            wtr.flush().unwrap();
        })
    });

    g.bench_function("bytes", |b| {
        b.iter(|| {
            let mut wtr = Writer::from_writer(vec![]);
            for r in &records {
                wtr.write_byte_record(r).unwrap();
            }
            wtr.flush().unwrap();
        })
    });

    g.finish();
}

/// A field whose `Deserialize` impl calls `deserialize_any`, so each field
/// goes through the deserializer's `infer_deserialize` cascade. Used to
/// benchmark the type-inference path that typed `Deserialize` derives skip.
#[derive(Debug, Default)]
struct InferredField;

impl<'de> Deserialize<'de> for InferredField {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(InferredVisitor)
    }
}

struct InferredVisitor;

impl<'de> Visitor<'de> for InferredVisitor {
    type Value = InferredField;

    fn expecting(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.write_str("any value")
    }

    fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E> {
        Ok(InferredField)
    }
    fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E> {
        Ok(InferredField)
    }
    fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E> {
        Ok(InferredField)
    }
    fn visit_i128<E>(self, _: i128) -> Result<Self::Value, E> {
        Ok(InferredField)
    }
    fn visit_u128<E>(self, _: u128) -> Result<Self::Value, E> {
        Ok(InferredField)
    }
    fn visit_f64<E>(self, _: f64) -> Result<Self::Value, E> {
        Ok(InferredField)
    }
    fn visit_str<E>(self, _: &str) -> Result<Self::Value, E> {
        Ok(InferredField)
    }
    fn visit_borrowed_str<E>(self, _: &'de str) -> Result<Self::Value, E> {
        Ok(InferredField)
    }
    fn visit_bytes<E>(self, _: &[u8]) -> Result<Self::Value, E> {
        Ok(InferredField)
    }
    fn visit_borrowed_bytes<E>(self, _: &'de [u8]) -> Result<Self::Value, E> {
        Ok(InferredField)
    }
}

// POP has 7 columns; this tuple matches the schema and exercises
// `infer_deserialize` once per field.
type POPInferredRow = (
    InferredField,
    InferredField,
    InferredField,
    InferredField,
    InferredField,
    InferredField,
    InferredField,
);

fn bench_pop_infer(c: &mut Criterion) {
    let data = POP.as_bytes();
    let mut g = c.benchmark_group("pop_infer");
    g.throughput(Throughput::Bytes(data.len() as u64));

    g.bench_function("infer_owned_str", |b| {
        b.iter(|| {
            let mut rdr =
                ReaderBuilder::new().has_headers(true).from_reader(data);
            let mut count = 0u64;
            for result in rdr.deserialize::<POPInferredRow>() {
                let _ = result.unwrap();
                count += 1;
            }
            assert_eq!(count, 20000);
        })
    });

    g.bench_function("infer_borrowed_bytes", |b| {
        b.iter(|| {
            let mut rdr =
                ReaderBuilder::new().has_headers(true).from_reader(data);
            let mut count = 0u64;
            let mut rec = ByteRecord::new();
            while rdr.read_byte_record(&mut rec).unwrap() {
                let _: POPInferredRow = rec.deserialize(None).unwrap();
                count += 1;
            }
            assert_eq!(count, 20000);
        })
    });

    g.finish();
}

criterion_group!(
    benches,
    bench_nfl,
    bench_nfl_trimmed,
    bench_nfl_write,
    bench_game,
    bench_pop,
    bench_pop_infer,
    bench_mbta,
);
criterion_main!(benches);
