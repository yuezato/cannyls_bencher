extern crate cannyls_bencher;
use cannyls_bencher::generator;
use cannyls_bencher::*;

fn main() {
    println!("こんにちわ🦀");

    let expected1 = Section {
        iter: 100,
        inner: SectionInner::Ordered(vec![
            (10, Command::RandomGet),
            (20, Command::Overwrite(43)),
            (30, Command::RandomDelete),
            (39, Command::Delete(10, 20)),
            (1, Command::DeleteRange(99, 100)),
        ]),
    };

    let expected2 = Section {
        iter: 100,
        inner: SectionInner::Unordered(vec![
            (30, Command::NewPut(128)),
            (30, Command::RandomGet),
            (20, Command::Overwrite(43)),
            (20, Command::RandomDelete),
        ]),
    };

    let w = Workload {
        sections: vec![expected2],
    };

    let mut state = generator::default_state();
    let commands = generator::deal_workload(&mut state, w);

    println!("{:?}", commands);

    generator::commands_to_real_commands(&mut state, commands);

    println!("{:?}", state.commands);;
}
