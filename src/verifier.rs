use crate::RealCommand;
use cannyls::lump;
use cannyls::nvm::NonVolatileMemory;
use cannyls::storage::Storage;
use std::ops::Range;

/*
LumpIdから128bit値を作り、
それを乱数のseedとして採用した上で
`size`バイトの配列を生成する。
*/
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
        RealCommand::Embed(lumpid, bytes) => {
            let v = lumpid_to_bytes(*lumpid, *bytes);
            let lump = lump::LumpData::new_embedded(v).unwrap();
            let _ = storage.put(&lumpid, &lump).unwrap();
        }
        RealCommand::Get(lumpid, bytes) => {
            let v = lumpid_to_bytes(*lumpid, *bytes);
            let lump = storage.get(&lumpid).unwrap();
            assert!(lump.is_some());

            let lump: lump::LumpData = lump.unwrap();

            if lump.as_bytes() != v.as_slice() {
                panic!("Get Error[Lumpid = {}]: Obtained data is invalid", lumpid);
            }
        }
        RealCommand::Delete(lumpid, bytes) => {
            // 削除前にデータを取得して検証を行う。
            let v = lumpid_to_bytes(*lumpid, *bytes);
            let lump = storage.get(&lumpid).unwrap();
            let lump: lump::LumpData = lump.unwrap();
            if lump.as_bytes() != v.as_slice() {
                panic!(
                    "Delete Error[Lumpid = {}]: Obtained data is invalid",
                    lumpid
                );
            }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflexitivity_of_lumpid_to_bytes() {
        let lumpid = lump::LumpId::new(0);
        assert_eq!(lumpid_to_bytes(lumpid, 512), lumpid_to_bytes(lumpid, 512));
        assert_eq!(
            lumpid_to_bytes(lumpid, 512).as_slice(),
            &lumpid_to_bytes(lumpid, 1024).as_slice()[0..512]
        );
    }

    #[test]
    fn size_of_lumpid_to_bytes() {
        let lumpid = lump::LumpId::new(0);
        assert_eq!(lumpid_to_bytes(lumpid, 512).len(), 512);
    }

    #[test]
    // This test should be failed with high possibility.
    fn lumpid_to_bytes_is_not_injective() {
        let lumpid0 = lump::LumpId::new(0);
        let lumpid1 = lump::LumpId::new(1);
        assert_ne!(lumpid_to_bytes(lumpid0, 512), lumpid_to_bytes(lumpid1, 512));
    }
}
