use ccs::lts::Lts;
use ccs::process::{Channel, Process};
use ccs::utils::print_bisimulation;

fn main() {
    let a = Process::constant("a");
    let b = Process::constant("b");
    let c = Process::constant("c");
    let d = Process::constant("d");
    let ch1 = Channel::send("ch1");
    let ch2 = Channel::send("ch2");

    let lts1 = Lts::from_array([(&a, &ch1, &b)]);
    let lts2 = Lts::from_array([(&c, &ch2, &d)]);

    let b = lts1.bisimilarity(&lts2);
    print_bisimulation(&b);
}
