use crate::{Bytes, RealCommand};
use cannyls::lump;
use cannyls::nvm::{FileNvm, NonVolatileMemory};
use cannyls::storage::{Storage, StorageBuilder};
use std::collections::BTreeMap;
use std::ops::Range;
use std::time::{Duration, Instant};

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord)]
enum CommandKind {
    Put(Bytes),

    Get(Bytes),

    Delete,

    DeleteRange,
}

pub struct Summary {
    result: BTreeMap<CommandKind, Vec<Duration>>,
    total_time: Duration,
}

impl Default for Summary {
    fn default() -> Summary {
        Summary {
            result: BTreeMap::new(),
            total_time: Duration::new(0, 0),
        }
    }
}

fn percentile(v: &[Duration], p: u8) -> Duration {
    // assert!(v.is_sorted());
    assert!(p <= 100);

    let pos: usize = (p as usize * v.len()) / 100;
    v[pos.saturating_sub(1)]
}

pub fn statistics(s: &mut Summary) {
    let mut overall = Vec::new();

    for (kind, v) in &mut s.result {
        v.sort();

        let p50 = percentile(v, 50);
        let p90 = percentile(v, 90);
        let p95 = percentile(v, 95);
        let p99 = percentile(v, 99);
        println!(
            "kind = {:?}, count = {}, 50% = {:?}, 90% = {:?}, 95% = {:?}, 99% = {:?}",
            kind,
            v.len(),
            p50,
            p90,
            p95,
            p99
        );

        overall.append(v);
    }

    {
        overall.sort();

        let p50 = percentile(&overall, 50);
        let p90 = percentile(&overall, 90);
        let p95 = percentile(&overall, 95);
        let p99 = percentile(&overall, 99);
        println!(
            "[Overall {}] 50% = {:?}, 90% = {:?}, 95% = {:?}, 99% = {:?}",
            overall.len(),
            p50,
            p90,
            p95,
            p99,
        );
        println!("Total Elapsed Time by Commands = {:?}", s.total_time);
    }
}

pub fn do_commands<N>(storage: &mut Storage<N>, commands: &[RealCommand]) -> Summary
where
    N: NonVolatileMemory,
{
    let mut summary: Summary = Default::default();

    for command in commands {
        do_command(storage, command, &mut summary)
    }

    summary
}

pub fn do_command<N>(storage: &mut Storage<N>, command: &RealCommand, summary: &mut Summary)
where
    N: NonVolatileMemory,
{
    match command {
        RealCommand::Put(lumpid, bytes) => {
            let lump = storage.allocate_lump_data(*bytes).unwrap();
            {
                let now = Instant::now();
                let _ = storage.put(&lumpid, &lump).unwrap();
                let elapsed = now.elapsed();

                summary.total_time += elapsed;

                if let Some(v) = summary.result.get_mut(&CommandKind::Put(*bytes)) {
                    v.push(elapsed);
                } else {
                    summary
                        .result
                        .insert(CommandKind::Put(*bytes), vec![elapsed]);
                }
            }
        }
        RealCommand::Get(lumpid, bytes) => {
            let now = Instant::now();
            let lump = storage.get(&lumpid).unwrap();
            let elapsed = now.elapsed();

            summary.total_time += elapsed;

            assert!(lump.is_some());

            let lump: lump::LumpData = lump.unwrap();

            if lump.as_bytes().len() != *bytes {
                panic!(
                    "GET Error[Lumpid = {}]: size = {}, expected size = {}",
                    lumpid,
                    lump.as_bytes().len(),
                    bytes
                );
            }

            if let Some(v) = summary.result.get_mut(&CommandKind::Get(*bytes)) {
                v.push(elapsed);
            } else {
                summary
                    .result
                    .insert(CommandKind::Get(*bytes), vec![elapsed]);
            }
        }
        RealCommand::Delete(lumpid, _) => {
            let now = Instant::now();
            let existed = storage.delete(&lumpid).unwrap();
            let elapsed = now.elapsed();

            summary.total_time += elapsed;

            if !existed {
                panic!("Delete Error: Lumpid = {} does not exist", lumpid);
            }

            if let Some(v) = summary.result.get_mut(&CommandKind::Delete) {
                v.push(elapsed);
            } else {
                summary.result.insert(CommandKind::Delete, vec![elapsed]);
            }
        }
        RealCommand::DeleteRange(start, end) => {
            let now = Instant::now();
            let _ = storage
                .delete_range(Range {
                    start: *start,
                    end: *end,
                })
                .unwrap();
            let elapsed = now.elapsed();

            summary.total_time += elapsed;

            if let Some(v) = summary.result.get_mut(&CommandKind::DeleteRange) {
                v.push(elapsed);
            } else {
                summary
                    .result
                    .insert(CommandKind::DeleteRange, vec![elapsed]);
            }
        }
    }
}

pub fn make_storage_on_file<P>(
    filepath: P,
    capacity: u64,
    safe_release_mode: bool,
) -> Storage<FileNvm>
where
    P: AsRef<std::path::Path>,
{
    let filenvm = FileNvm::create(filepath, capacity).unwrap();
    let mut builder = StorageBuilder::new();
    if safe_release_mode {
        builder.enable_safe_release_mode();
    }
    builder.create(filenvm).unwrap()
}
