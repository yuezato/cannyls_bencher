use crate::{Bytes, RealCommand};
use cannyls::nvm::{FileNvm, NonVolatileMemory};
use cannyls::storage::{Storage, StorageBuilder};
use std::collections::BTreeMap;
use std::ops::Range;
use std::time::{Duration, Instant};

#[derive(PartialOrd, PartialEq, Eq, Ord)]
enum CommandKind {
    Put(Bytes),

    Get(Bytes),

    Delete(Bytes),

    DeleteRange,
}

pub struct History(BTreeMap<CommandKind, Vec<Duration>>);

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
        RealCommand::Get(lumpid) => {
            let _ = storage.get(&lumpid).unwrap();
        }
        RealCommand::Delete(lumpid) => {
            let _ = storage.delete(&lumpid).unwrap();
        }
        RealCommand::DeleteRange(start, end) => {
            let _ = storage
                .delete_range(Range {
                    start: *start,
                    end: *end,
                })
                .unwrap();
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
