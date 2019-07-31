extern crate cannyls_bencher;
use cannyls_bencher::generator;
use cannyls_bencher::*;

fn main() {
    println!("🦀");

    let section = Section {
        iter: 100,
        inner: SectionInner::Unordered(vec![
            (30, Command::NewPut(128)),
            (30, Command::RandomGet),
            (20, Command::Overwrite(43)),
            (20, Command::RandomDelete),
        ]),
    };

    let w = Workload {
        seed: None,
        sections: vec![section],
    };

    let commands = generator::workload_to_real_commands(&w);

    println!("{:?}", commands);;
}
