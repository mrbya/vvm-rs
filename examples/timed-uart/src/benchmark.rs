//! Hidden native benchmark harness for the timed UART example.

use vvm::coverage::{Bin, Coverpoint, Cross2};
use vvm::random::{RandomContext, ReplayToken, Seed};
use vvm::timing::{SimulationTime, TimingScheduler};

use crate::timed_uart::TimedUart;

/// Stable replay stream for timed-UART benchmark traffic.
pub const BENCH_REPLAY: ReplayToken = ReplayToken::new(Seed::new(0x75_a4_21_9c));
/// Number of frames used by the benchmark workload.
pub const BENCH_FRAMES: u8 = 24;

/// Timed UART benchmark error.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParityMode {
    Disabled,
    Even,
    Odd,
}

#[derive(Clone, Copy, Debug)]
struct UartRequest {
    data: u8,
    parity: ParityMode,
    inject_parity_error: bool,
    inject_stop_error: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Frame {
    data: u8,
    parity: Option<bool>,
    stop: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Transition {
    time: SimulationTime,
    tx: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ErrorState {
    None,
    Parity,
    Stop,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DataClass {
    Zero,
    Maximum,
    Alternating,
    Other,
}

/// Runs deterministic timed-UART traffic with protocol reconstruction.
pub fn run_randomized() -> Result<()> {
    let mut random = RandomContext::from_replay(BENCH_REPLAY);

    for index in 0..BENCH_FRAMES {
        let request = random_request(index, &mut random);
        let frame = verify_frame(request)?;
        assert_eq!(frame, expected_frame(request));
    }

    Ok(())
}

/// Runs the same deterministic timed-UART traffic while sampling manual coverage.
pub fn run_randomized_with_coverage() -> Result<()> {
    let mut random = RandomContext::from_replay(BENCH_REPLAY);
    let parity_mode = Coverpoint::builder("parity_mode")
        .bin(Bin::value("disabled", ParityMode::Disabled))
        .bin(Bin::value("even", ParityMode::Even))
        .bin(Bin::value("odd", ParityMode::Odd))
        .build()?;
    let error_state = Coverpoint::builder("error_state")
        .bin(Bin::value("none", ErrorState::None))
        .bin(Bin::value("parity", ErrorState::Parity))
        .bin(Bin::value("stop", ErrorState::Stop))
        .bin(Bin::value("both", ErrorState::Both))
        .build()?;
    let data_class = Coverpoint::builder("data_class")
        .bin(Bin::value("zero", DataClass::Zero))
        .bin(Bin::value("maximum", DataClass::Maximum))
        .bin(Bin::value("alternating", DataClass::Alternating))
        .bin(Bin::value("other", DataClass::Other))
        .build()?;
    let mut parity_mode = parity_mode;
    let mut error_state = error_state;
    let mut data_class = data_class;
    let mut parity_x_error =
        Cross2::builder("parity_x_error", &parity_mode, &error_state).build()?;

    for index in 0..BENCH_FRAMES {
        let request = random_request(index, &mut random);
        let frame = verify_frame(request)?;
        assert_eq!(frame, expected_frame(request));

        let parity_sample = parity_mode.sample(&request.parity)?;
        let error_sample = error_state.sample(&observed_errors(request, frame))?;
        data_class.sample(&classify_data(request.data))?;
        parity_x_error.sample(&parity_sample, &error_sample)?;
    }

    Ok(())
}

fn verify_frame(request: UartRequest) -> Result<Frame> {
    let (transitions, mut dut) = run(request)?;
    let frame = decode(request, &transitions);

    assert!(dut.done()?);
    assert!(!dut.busy()?);
    assert!(dut.tx()?);

    dut.finish()?;

    Ok(frame)
}

fn run(request: UartRequest) -> Result<(Vec<Transition>, TimedUart)> {
    let mut dut = TimedUart::new()?;
    dut.set_data(request.data)?;
    dut.set_parity_enable(request.parity != ParityMode::Disabled)?;
    dut.set_odd_parity(request.parity == ParityMode::Odd)?;
    dut.set_inject_parity_error(request.inject_parity_error)?;
    dut.set_inject_stop_error(request.inject_stop_error)?;

    let mut scheduler = TimingScheduler::new();
    scheduler.initialize(&mut dut)?;
    let mut transitions = vec![Transition {
        time: SimulationTime::ZERO,
        tx: dut.tx()?,
    }];

    while let Some(event) = scheduler.advance_next(&mut dut)? {
        transitions.push(Transition {
            time: event.time(),
            tx: dut.tx()?,
        });
    }

    Ok((transitions, dut))
}

fn decode(request: UartRequest, transitions: &[Transition]) -> Frame {
    let levels = transitions
        .iter()
        .map(|transition| transition.tx)
        .collect::<Vec<_>>();
    let data = levels
        .get(2..10)
        .unwrap_or_default()
        .iter()
        .enumerate()
        .fold(0_u8, |value, (bit, level)| {
            value | (u8::from(*level) << bit)
        });
    let parity_enabled = request.parity != ParityMode::Disabled;
    let parity = parity_enabled.then(|| levels.get(10).copied().unwrap_or_default());
    let stop_index = 10_usize.saturating_add(usize::from(parity_enabled));

    Frame {
        data,
        parity,
        stop: levels.get(stop_index).copied().unwrap_or_default(),
    }
}

fn expected_frame(request: UartRequest) -> Frame {
    let parity = (request.parity != ParityMode::Disabled).then(|| {
        (!request.data.count_ones().is_multiple_of(2))
            ^ (request.parity == ParityMode::Odd)
            ^ request.inject_parity_error
    });

    Frame {
        data: request.data,
        parity,
        stop: !request.inject_stop_error,
    }
}

fn observed_errors(request: UartRequest, frame: Frame) -> ErrorState {
    let uncorrupted = expected_frame(UartRequest {
        inject_parity_error: false,
        inject_stop_error: false,
        ..request
    });

    match (
        frame.parity != uncorrupted.parity,
        frame.stop != uncorrupted.stop,
    ) {
        (false, false) => ErrorState::None,
        (true, false) => ErrorState::Parity,
        (false, true) => ErrorState::Stop,
        (true, true) => ErrorState::Both,
    }
}

const fn classify_data(data: u8) -> DataClass {
    match data {
        0x00 => DataClass::Zero,
        u8::MAX => DataClass::Maximum,
        0x55 | 0xaa => DataClass::Alternating,
        _ => DataClass::Other,
    }
}

fn random_request(index: u8, random: &mut RandomContext) -> UartRequest {
    let data = match index {
        0 => 0x00,
        1 => u8::MAX,
        2 => 0x55,
        3 => 0xaa,
        _ => u8::try_from(random.next_u32() & 0xff).unwrap_or_default(),
    };
    let parity = match if index < 12 {
        u32::from(index % 3)
    } else {
        random.next_u32() % 3
    } {
        0 => ParityMode::Disabled,
        1 => ParityMode::Even,
        _ => ParityMode::Odd,
    };
    let error_bits = if index < 12 {
        const DIRECTED_ERROR_BITS: [u8; 12] = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3];
        DIRECTED_ERROR_BITS
            .get(usize::from(index))
            .copied()
            .unwrap_or_default()
    } else {
        u8::try_from(random.next_u32() & 0b11).unwrap_or_default()
    };

    UartRequest {
        data,
        parity,
        inject_parity_error: error_bits & 1 != 0,
        inject_stop_error: error_bits & 2 != 0,
    }
}
