extern crate cannyls_bencher;
extern crate fibers_global;
extern crate fibers_http_server;
extern crate futures;
use cannyls_bencher::generator;
use cannyls_bencher::*;
use futures::{lazy, Future};

use fibers_http_server::metrics::{MetricsHandler, WithMetrics};
use fibers_http_server::ServerBuilder;

use chrono::Local;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cannyls_bencher🦀")]
struct Opt {
    #[structopt(long)]
    workload: PathBuf,

    #[structopt(long, parse(try_from_str = "parse_with_suffix"))]
    capacity: u64,

    #[structopt(long)]
    lusfname: PathBuf,
}

fn parse_with_suffix(s: &str) -> Result<u64, String> {
    use combine::parser::Parser;

    if let Ok((num, rest)) = parse::parse_bytes_with_suffix().parse(s) {
        if rest.is_empty() {
            Ok(num as u64)
        } else {
            Err("Fail".to_owned())
        }
    } else {
        Err("Fail".to_owned())
    }
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
    let addr = "0.0.0.0:5555".parse().unwrap();
    let mut builder = ServerBuilder::new(addr);
    builder
        .add_handler(WithMetrics::new(MetricsHandler))
        .unwrap();

    let server = builder.finish(fibers_global::handle());
    fibers_global::spawn(server.map_err(|_| ()));

    let opt = Opt::from_args();
    let capacity = opt.capacity;
    let lusfname = opt.lusfname.clone();
    println!("{:#?}", opt);

    let w = file_to_workload(opt.workload);

    fibers_global::execute(
        lazy(move || {
            let commands = generator::workload_to_real_commands(&w);

            let mut storage = run_commands::make_storage_on_file(lusfname, capacity);

            println!("Start Benchmark @ {}", Local::now());
            let mut summary = run_commands::do_commands(&mut storage, &commands);
            println!("Finish Benchmark @ {}", Local::now());

            println!("Calculating Statistics...");
            run_commands::statistics(&mut summary);

            Ok(())
        })
        .map(|_| ())
        .map_err(|_: ()| ()),
    )
    .unwrap();
}
