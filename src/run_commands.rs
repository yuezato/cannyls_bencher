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

pub struct History(BTreeMap<CommandKind, Vec<Duration>>);

fn percentile(v: &[Duration], p: u8) -> Duration {
    // assert!(v.is_sorted());
    assert!(p <= 100);

    let pos: usize = (p as usize * v.len()) / 100;
    v[pos.saturating_sub(1)]
}

pub fn statistics(h: &mut History) {
    let mut overall = Vec::new();

    for (kind, v) in &mut h.0 {
        v.sort();

        let p50 = percentile(v, 50);
        let p90 = percentile(v, 90);
        let p95 = percentile(v, 95);
        let p99 = percentile(v, 99);
        println!(
            "kind = {:?}, 50% = {:?}, 90% = {:?}, 95% = {:?}, 99% = {:?}",
            kind, p50, p90, p95, p99
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
            p99
        );
    }
}

pub fn do_commands<N>(storage: &mut Storage<N>, commands: &[RealCommand]) -> History
where
    N: NonVolatileMemory,
{
    let mut history: History = History(BTreeMap::new());

    for command in commands {
        do_command(storage, command, &mut history)
    }

    history
}

pub fn do_command<N>(storage: &mut Storage<N>, command: &RealCommand, history: &mut History)
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

                if let Some(v) = history.0.get_mut(&CommandKind::Put(*bytes)) {
                    v.push(elapsed);
                } else {
                    history.0.insert(CommandKind::Put(*bytes), vec![elapsed]);
                }
            }
        }
        RealCommand::Get(lumpid, bytes) => {
            let now = Instant::now();
            let lump = storage.get(&lumpid).unwrap();
            let elapsed = now.elapsed();

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

            if let Some(v) = history.0.get_mut(&CommandKind::Get(*bytes)) {
                v.push(elapsed);
            } else {
                history.0.insert(CommandKind::Get(*bytes), vec![elapsed]);
            }
        }
        RealCommand::Delete(lumpid) => {
            let now = Instant::now();
            let existed = storage.delete(&lumpid).unwrap();
            let elapsed = now.elapsed();

            if !existed {
                panic!("Delete Error: Lumpid = {} does not exist", lumpid);
            }

            if let Some(v) = history.0.get_mut(&CommandKind::Delete) {
                v.push(elapsed);
            } else {
                history.0.insert(CommandKind::Delete, vec![elapsed]);
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

            if let Some(v) = history.0.get_mut(&CommandKind::DeleteRange) {
                v.push(elapsed);
            } else {
                history.0.insert(CommandKind::DeleteRange, vec![elapsed]);
            }
        }
    }
}

pub fn make_storage_on_file<P>(filepath: P, capacity: u64) -> Storage<FileNvm>
where
    P: AsRef<std::path::Path>,
{
    let filenvm = FileNvm::create(filepath, capacity).unwrap();
    let builder = StorageBuilder::new();
    builder.create(filenvm).unwrap()
}
