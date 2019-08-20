extern crate cannyls;
extern crate rand;
pub use cannyls::lump::LumpId;

pub mod generator;
pub mod parse;
pub mod run_commands;

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

#[derive(Clone, Debug, PartialEq)]
pub struct Statement(Vec<Command>);

#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub inner: SectionInner,
    pub iter: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SectionInner {
    Ordered(Vec<(Freq, Statement)>),
    Unordered(Vec<(Freq, Statement)>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Workload {
    pub seed: Option<u64>,
    pub sections: Vec<Section>,
}

#[derive(Debug, PartialEq)]
pub enum RealCommand {
    // Put
    Put(LumpId, Bytes),

    // Get
    Get(LumpId, Bytes),

    // Delete
    Delete(LumpId),

    // DeleteRange
    DeleteRange(LumpId, LumpId),
}
