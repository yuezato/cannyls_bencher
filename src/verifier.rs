use crate::RealCommand;
use cannyls::lump;
use cannyls::nvm::NonVolatileMemory;
use cannyls::storage::Storage;
use std::ops::Range;

pub fn lumpid_to_bytes(lumpid: lump::LumpId, size: usize) -> Vec<u8> {
    use rand::rngs::StdRng;
    use rand::{RngCore, SeedableRng};

    let mut v = vec![0; size];
    let lumpid: u128 = lumpid.as_u128();
    let mut rng: StdRng = {
        let mut tmp = [0; 32];
        tmp[16..].copy_from_slice(&lumpid.to_be_bytes());
        StdRng::from_seed(tmp)
    };
    rng.fill_bytes(&mut v);
    v
}

pub fn verify_commands<N>(storage: &mut Storage<N>, commands: &[RealCommand])
where
    N: NonVolatileMemory,
{
    for command in commands {
        verify_command(storage, command)
    }
}

pub fn verify_command<N>(storage: &mut Storage<N>, command: &RealCommand)
where
    N: NonVolatileMemory,
{
    match command {
        RealCommand::Put(lumpid, bytes) => {
            let v = lumpid_to_bytes(*lumpid, *bytes);
            let lump = storage.allocate_lump_data_with_bytes(&v).unwrap();
            let _ = storage.put(&lumpid, &lump).unwrap();
        }
        RealCommand::Get(lumpid, bytes) => {
            let v = lumpid_to_bytes(*lumpid, *bytes);
            let lump = storage.get(&lumpid).unwrap();
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
            if lump.as_bytes() == v.as_slice() {
                panic!("Get Error[Lumpid = {}]: Obtained data is invalid", lumpid);
            }
        }
        RealCommand::Delete(lumpid) => {
            let existed = storage.delete(&lumpid).unwrap();
            if !existed {
                panic!("Delete Error: Lumpid = {} does not exist", lumpid);
            }
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
