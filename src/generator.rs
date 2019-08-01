use super::{Bytes, RealCommand, Workload};
use crate::rand::SeedableRng;
use crate::{Command, Section, SectionInner};
use cannyls::lump::LumpId;

pub struct State {
    rng: rand::rngs::StdRng,
    next: LumpId,
    live_ids: Vec<(LumpId, Bytes)>,
    pub commands: Vec<RealCommand>,
}

impl State {
    pub fn new(seed: Option<u64>) -> State {
        State {
            rng: rand::rngs::StdRng::seed_from_u64(seed.unwrap_or(0)),
            next: LumpId::new(1),
            live_ids: Vec::new(),
            commands: Vec::new(),
        }
    }
}

pub fn workload_to_real_commands(workload: &Workload) -> Vec<RealCommand> {
    let mut state = State::new(workload.seed);
    let commands = deal_workload(&mut state, workload);
    commands_to_real_commands(&mut state, commands);
    state.commands
}

pub fn commands_to_real_commands(state: &mut State, commands: Vec<Command>) {
    for command in commands {
        match command {
            Command::NewPut(bytes) => put(state, bytes),
            Command::Overwrite(bytes) => overwrite(state, bytes),
            Command::RandomGet => get(state, 0, 100),
            Command::Get(left, right) => get(state, left, right),
            Command::RandomDelete => delete(state, 0, 100),
            Command::Delete(left, right) => delete(state, left, right),
            Command::DeleteRange(left, right) => delete_range(state, left, right),
        }
    }
}

pub fn default_state() -> State {
    State {
        rng: rand::rngs::StdRng::seed_from_u64(0),
        next: LumpId::new(0),
        live_ids: Vec::new(),
        commands: Vec::new(),
    }
}

pub fn deal_workload(state: &mut State, workload: &Workload) -> Vec<Command> {
    let mut commands: Vec<Command> = Vec::new();

    for section in &workload.sections {
        commands.append(&mut section_to_commands(state, section));
    }

    commands
}

fn section_to_commands(state: &mut State, section: &Section) -> Vec<Command> {
    use rand::seq::SliceRandom;

    let v = match &section.inner {
        SectionInner::Ordered(v) => v,
        SectionInner::Unordered(v) => v,
    };

    let mut commands: Vec<Command> = Vec::new();

    for (freq, command) in v {
        let y = (section.iter * *freq as usize) / 100;
        commands.append(&mut vec![command.clone(); y]);
    }

    if let SectionInner::Unordered(_) = section.inner {
        commands.shuffle(&mut state.rng);
    }

    commands
}

fn put(state: &mut State, bytes: Bytes) {
    let lumpid = state.next.as_u128();
    state.next = LumpId::new(lumpid + 1);
    let lumpid: LumpId = LumpId::new(lumpid);
    state.live_ids.push((lumpid, bytes));
    state.commands.push(RealCommand::Put(lumpid, bytes));
}

fn overwrite(state: &mut State, bytes: Bytes) {
    if state.live_ids.is_empty() {
        return;
    }
    let z = choose(&mut state.rng, 0, state.live_ids.len() - 1);
    let lumpid = state.live_ids[z].0;
    state.commands.push(RealCommand::Put(lumpid, bytes));
}

fn get(state: &mut State, left: u8, right: u8) {
    if state.live_ids.is_empty() {
        return;
    }
    let l = state.live_ids.len().saturating_sub(1);
    let x = (l * left as usize) / 100;
    let y = (l * right as usize) / 100;
    let z = choose(&mut state.rng, x, y);
    let (lumpid, bytes) = state.live_ids[z];
    state.commands.push(RealCommand::Get(lumpid, bytes));
}

fn delete(state: &mut State, left: u8, right: u8) {
    if state.live_ids.is_empty() {
        return;
    }
    let l = state.live_ids.len().saturating_sub(1);
    let x = (l * left as usize) / 100;
    let y = (l * right as usize) / 100;
    let z = choose(&mut state.rng, x, y);
    let lumpid = state.live_ids[z].0;
    state.commands.push(RealCommand::Delete(lumpid));
    state.live_ids.remove(z);
}

fn delete_range(state: &mut State, left: u8, right: u8) {
    if state.live_ids.is_empty() {
        return;
    }
    let l = state.live_ids.len().saturating_sub(1);
    let x = (l * left as usize) / 100;
    let y = (l * right as usize) / 100;
    let lumpid1 = state.live_ids[x].0;
    let lumpid2 = state.live_ids[y].0;
    state
        .commands
        .push(RealCommand::DeleteRange(lumpid1, lumpid2));
    state.live_ids.drain(x..=y);
}

fn choose<R>(rng: &mut R, start: usize, end: usize) -> usize
where
    R: rand::Rng,
{
    rng.gen_range(start, end)
}
