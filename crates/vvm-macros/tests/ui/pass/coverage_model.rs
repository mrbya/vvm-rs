use vvm::{coverage::{Bin, CoverageBuildError, Coverpoint, Cross2}, testbench::ObservedCycle};

#[derive(vvm::Coverage)]
#[vvm(definition = "model", revision = 1, stimulus = Stimulus, observation = Observation)]
struct Model {
    #[vvm(coverpoint(build = left, sample = sample_left))]
    left: Coverpoint<u8>,
    #[vvm(coverpoint(build = right, sample = sample_right))]
    right: Coverpoint<bool>,
    #[vvm(cross(left = left, right = right))]
    cross: Cross2,
}

#[derive(Clone, Copy)]
struct Stimulus(u8);
#[derive(Clone, Copy)]
struct Observation(bool);

fn left(name: &'static str) -> Result<Coverpoint<u8>, CoverageBuildError> {
    Coverpoint::builder(name).bin(Bin::value("zero", 0)).build()
}

fn right(name: &'static str) -> Result<Coverpoint<bool>, CoverageBuildError> {
    Coverpoint::builder(name).bin(Bin::value("false", false)).build()
}

fn sample_left(cycle: ObservedCycle<'_, Stimulus, Observation>) -> u8 { cycle.stimulus().0 }
fn sample_right(cycle: ObservedCycle<'_, Stimulus, Observation>) -> bool { cycle.observed().0 }

fn main() { let _ = Model::new("dut.model"); }
