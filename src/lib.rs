extern crate cannyls;
extern crate rand;
pub use cannyls::lump::LumpId;

pub mod generator;
pub mod parse;

pub type Bytes = usize;
pub type Perc = u8;
pub type Freq = u8;

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    // Put
    NewPut(Bytes),
    Overwrite(Bytes),

    // Get
    RandomGet,
    Get(Perc, Perc),

    // Delete
    RandomDelete,
    Delete(Perc, Perc),

    // DeleteRange
    DeleteRange(Perc, Perc),
}

#[derive(Debug, PartialEq)]
pub struct Section {
    pub inner: SectionInner,
    pub iter: usize,
}

#[derive(Debug, PartialEq)]
pub enum SectionInner {
    Ordered(Vec<(Freq, Command)>),
    Unordered(Vec<(Freq, Command)>),
}

#[derive(Debug, PartialEq)]
pub struct Workload {
    pub sections: Vec<Section>,
}

#[derive(Debug, PartialEq)]
pub enum RealCommand {
    // Put
    Put(LumpId, Bytes),

    // Get
    Get(LumpId),

    // Delete
    Delete(LumpId),

    // DeleteRange
    DeleteRange(LumpId, LumpId),
}
