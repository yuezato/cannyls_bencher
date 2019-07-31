use crate::RealCommand;
use cannyls::nvm::{FileNvm, NonVolatileMemory};
use cannyls::storage::{Storage, StorageBuilder};
use std::ops::Range;

pub fn do_commands<N>(storage: &mut Storage<N>, commands: &[RealCommand])
where
    N: NonVolatileMemory,
{
    for command in commands {
        do_command(storage, command)
    }
}

pub fn do_command<N>(storage: &mut Storage<N>, command: &RealCommand)
where
    N: NonVolatileMemory,
{
    match command {
        RealCommand::Put(lumpid, bytes) => {
            let lump = storage.allocate_lump_data(*bytes).unwrap();
            let _ = storage.put(&lumpid, &lump).unwrap();
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
