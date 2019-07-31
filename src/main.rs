extern crate cannyls_bencher;
use cannyls_bencher::generator;
use cannyls_bencher::*;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cannyls_bencher🦀")]
struct Opt {
    #[structopt(long)]
    workload: PathBuf,

    #[structopt(long)]
    capacity: u64,

    #[structopt(long)]
    lusfname: PathBuf,
}

fn file_to_workload<P: AsRef<std::path::Path>>(filepath: P) -> Workload {
    use combine::parser::Parser;

    let workload: String = std::fs::read_to_string(filepath).unwrap();
    let workload: Workload = parse::parse_workload()
        .parse(workload.as_ref() as &str)
        .unwrap()
        .0;

    workload
}

fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);

    let w = file_to_workload(opt.workload);
    let commands = generator::workload_to_real_commands(&w);

    println!("{:?}", commands);

    let mut storage = run_commands::make_storage_on_file(opt.lusfname, opt.capacity);
    run_commands::do_commands(&mut storage, &commands);
}
