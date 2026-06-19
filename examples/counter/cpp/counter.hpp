#pragma once

#include <cstdint>
#include <memory>

namespace vvm::counter {

/// Owns and provides access to a Verilated counter model.
class Counter final {
public:
    /// Constructs a Verilated counter model.
    Counter();

    /// Finalises and destroys the Verilated counter model.
    ~Counter() noexcept;

    Counter(const Counter&) = delete;
    Counter& operator=(const Counter&) = delete;

    Counter(Counter&&) = delete;
    Counter& operator=(Counter&&) = delete;

    /// Evaluates the Verilated model.
    void eval() noexcept;

    /// Finalises the Verilated model.
    ///
    /// Calling this function more than once has no effect.
    void finish() noexcept;

    /// Drives the clock input.
    void set_clk(bool value) noexcept;

    /// Drives the active-low reset input.
    void set_reset_n(bool value) noexcept;

    /// Drives the counter-enable input.
    void set_enable(bool value) noexcept;

    /// Samples the counter output.
    [[nodiscard]] std::uint8_t count() const noexcept;

private:
    class Impl;

    std::unique_ptr<Impl> impl_;
};

/// Constructs a counter model.
///
/// Returns a null pointer when model construction fails.
[[nodiscard]] std::unique_ptr<Counter> create_counter() noexcept;

} // namespace vvm::counter
