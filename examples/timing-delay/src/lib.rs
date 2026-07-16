//! Timing-enabled delayed-event verification example.

#[cfg(test)]
vvm::include_dut!(delayed_sequence);

#[cfg(test)]
mod verification;

#[cfg(test)]
mod tests {
    use vvm::{Dut, SimulationTime, TimedDut, TimingScheduler};

    use crate::delayed_sequence::DelayedSequence;
    use crate::verification::{
        Result, parse_vcd_timestamps, require_event, time_slot_limit, timestamps_increase,
    };

    /// Compile-time assertion for timing-enabled generated wrappers.
    fn assert_timed_dut<D>()
    where
        D: TimedDut,
    {
    }

    #[test]
    fn manually_steps_delayed_events() -> Result<()> {
        assert_timed_dut::<DelayedSequence>();

        let mut dut = DelayedSequence::new()?;
        let mut scheduler = TimingScheduler::new();

        scheduler.initialize(&mut dut)?;

        assert_eq!(scheduler.start_time(), Some(SimulationTime::ZERO));
        assert_eq!(scheduler.time_slots(), 0);
        assert_eq!(scheduler.evaluations(), 1);
        assert_eq!(dut.simulation_time(), SimulationTime::ZERO);
        assert_eq!(dut.value()?, 0x00);
        assert!(!dut.done()?);
        assert!(dut.events_pending()?);
        assert_eq!(dut.next_time_slot()?, Some(SimulationTime::from_ticks(2)));

        let first = require_event(scheduler.advance_next(&mut dut)?, 0)?;

        assert_eq!(first.ordinal(), 0);
        assert_eq!(first.time(), SimulationTime::from_ticks(2));
        assert_eq!(first.elapsed().ticks(), 2);
        assert_eq!(dut.value()?, 0x11);
        assert!(!dut.done()?);
        assert_eq!(dut.next_time_slot()?, Some(SimulationTime::from_ticks(5)));

        let second = require_event(scheduler.advance_next(&mut dut)?, 1)?;

        assert_eq!(second.ordinal(), 1);
        assert_eq!(second.time(), SimulationTime::from_ticks(5));
        assert_eq!(second.elapsed().ticks(), 3);
        assert_eq!(dut.value()?, 0x22);
        assert!(!dut.done()?);
        assert_eq!(dut.next_time_slot()?, Some(SimulationTime::from_ticks(10)));

        let third = require_event(scheduler.advance_next(&mut dut)?, 2)?;

        assert_eq!(third.ordinal(), 2);
        assert_eq!(third.time(), SimulationTime::from_ticks(10));
        assert_eq!(third.elapsed().ticks(), 5);
        assert_eq!(dut.value()?, 0x33);
        assert!(dut.done()?);
        assert!(!dut.events_pending()?);
        assert_eq!(dut.next_time_slot()?, None);
        assert!(scheduler.advance_next(&mut dut)?.is_none());
        assert_eq!(scheduler.time_slots(), 3);
        assert_eq!(scheduler.evaluations(), 4);
        assert_eq!(dut.simulation_time(), SimulationTime::from_ticks(10));
        assert!(!dut.is_finished());

        dut.finish()?;

        assert!(dut.is_finished());

        Ok(())
    }

    #[test]
    fn runs_delayed_model_until_idle() -> Result<()> {
        let mut dut = DelayedSequence::new()?;
        let mut scheduler = TimingScheduler::new();

        let run = scheduler.run_until_idle(&mut dut, time_slot_limit()?)?;

        assert!(scheduler.is_initialized());
        assert_eq!(run.start_time(), SimulationTime::ZERO);
        assert_eq!(run.final_time(), SimulationTime::from_ticks(10));
        assert_eq!(run.time_slots(), 3);
        assert_eq!(run.evaluations(), 4);
        assert_eq!(scheduler.time_slots(), run.time_slots());
        assert_eq!(scheduler.evaluations(), run.evaluations());
        assert_eq!(dut.simulation_time(), SimulationTime::from_ticks(10));
        assert_eq!(dut.value()?, 0x33);
        assert!(dut.done()?);
        assert!(!dut.events_pending()?);
        assert!(!dut.is_finished());

        dut.finish()?;

        assert!(dut.is_finished());

        Ok(())
    }

    #[test]
    fn generates_delayed_event_vcd() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let trace_path = directory.path().join("delayed_sequence.vcd");
        let mut dut = DelayedSequence::new()?;

        assert!(!dut.trace_is_open());

        dut.open_trace(&trace_path)?;

        assert!(dut.trace_is_open());

        let mut scheduler = TimingScheduler::new();
        let run = scheduler.run_until_idle(&mut dut, time_slot_limit()?)?;

        assert_eq!(run.final_time(), SimulationTime::from_ticks(10));
        assert!(dut.trace_is_open());

        dut.finish()?;

        assert!(!dut.trace_is_open());

        let metadata = std::fs::metadata(&trace_path)?;

        assert!(metadata.is_file());
        assert!(metadata.len() > 0);

        let contents = std::fs::read_to_string(&trace_path)?;

        assert!(contents.contains("$enddefinitions"));
        assert!(contents.contains("$var"));
        assert!(contents.contains("value"));
        assert!(contents.contains("done"));

        let timestamps = parse_vcd_timestamps(&contents)?;

        assert_eq!(timestamps, vec![0, 2, 5, 10]);
        assert!(timestamps_increase(&timestamps));

        Ok(())
    }
}
