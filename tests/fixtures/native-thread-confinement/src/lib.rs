#[cfg(test)]
vvm::include_dut!(counter);

#[cfg(test)]
mod tests {
    use crate::counter::Counter;

    fn require_send<T: Send>() {}

    fn require_sync<T: Sync>() {}

    #[test]
    fn generated_counter_is_not_thread_safe() {
        require_send::<Counter>();
        require_sync::<Counter>();
    }
}
