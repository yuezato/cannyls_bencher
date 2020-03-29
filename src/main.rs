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

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "cannyls_bencher🦀")]
struct Opt {
    #[structopt(long)]
    workload: PathBuf,

    #[structopt(long, parse(try_from_str = "parse_with_suffix"))]
    capacity: Option<u64>,

    #[structopt(long)]
    lusfname: PathBuf,

    #[structopt(long)]
    verbose: bool,

    #[structopt(long)]
    verify_mode: bool,

    #[structopt(long)]
    block_size: Option<u16>,
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
    let opt = Opt::from_args();
    let lusfname = opt.lusfname.clone();
    let capacity = opt.capacity;
    let verify_mode = opt.verify_mode;
    let block_size = opt.block_size;
    println!("{:#?}", opt);

    let w = file_to_workload(opt.workload);

    if opt.verbose {
        println!("{:?}", w);
    }

    println!("Start Generating Commands @ {}", Local::now());
    let (commands, least_required) = generator::workload_to_real_commands(&w);
    println!("Finish Generating Commands @ {}", Local::now());
    println!("Least Required Bytes = {}", least_required);

    let mut storage = if let Some(capacity) = capacity {
        run_commands::make_storage_on_file(lusfname, capacity, block_size)
    } else {
        let mbyte = 1024 * 1024;
        let least_required = ((least_required + (mbyte - 1)) / mbyte) * mbyte;
        run_commands::make_storage_on_file(
            lusfname,
            (1.5 * least_required as f64) as u64,
            block_size,
        )
    };

    if verify_mode {
        println!("Start Verifying @ {}", Local::now());
        verifier::verify_commands(&mut storage, &commands);
        println!("Finish Verifying @ {}", Local::now());
        return;
    }

    // ベンチモードではCannylsのメトリクスを取れるようにする
    let addr = "0.0.0.0:5555".parse().unwrap();
    let mut builder = ServerBuilder::new(addr);
    builder
        .add_handler(WithMetrics::new(MetricsHandler))
        .unwrap();

    let server = builder.finish(fibers_global::handle());
    fibers_global::spawn(server.map_err(|e| panic!("Metrics Server Error: {:?}", e)));
    fibers_global::execute(
        lazy(move || {
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
