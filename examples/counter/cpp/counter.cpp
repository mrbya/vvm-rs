#include "counter.hpp"

#include "Vcounter.h"
#include "verilated.h"

#include <cstdint>
#include <memory>

namespace vvm::counter {

/// Private implementation of the counter adapter.
class Counter::Impl final {
public:
    /// Constructs the Verilator context and counter model.
    Impl()
        : context{std::make_unique<VerilatedContext>()},
          model{std::make_unique<Vcounter>(context.get())} {
        // Verilator does not guarantee useful initial values for model
        // inputs, so initialise every input explicitly.
        model->clk = 0;
        model->reset_n = 0;
        model->enable = 0;
    }

    /// Verilator simulation context.
    std::unique_ptr<VerilatedContext> context;

    /// Verilated counter model.
    std::unique_ptr<Vcounter> model;

    /// Whether the model has already been finalised.
    bool finished{false};
};

Counter::Counter()
    : impl_{std::make_unique<Impl>()} {}

Counter::~Counter() noexcept {
    finish();
}

void Counter::eval() noexcept {
    if (!impl_->finished) {
        impl_->model->eval();
    }
}

void Counter::finish() noexcept {
    if (impl_->finished) {
        return;
    }

    // Mark the model as finished before calling final() so that the
    // destructor can never finalise it a second time.
    impl_->finished = true;
    impl_->model->final();
}

void Counter::set_clk(const bool value) noexcept {
    impl_->model->clk = static_cast<std::uint8_t>(value);
}

void Counter::set_reset_n(const bool value) noexcept {
    impl_->model->reset_n = static_cast<std::uint8_t>(value);
}

void Counter::set_enable(const bool value) noexcept {
    impl_->model->enable = static_cast<std::uint8_t>(value);
}

std::uint8_t Counter::count() const noexcept {
    return static_cast<std::uint8_t>(impl_->model->count);
}

std::unique_ptr<Counter> create_counter() noexcept {
    try {
        return std::make_unique<Counter>();
    } catch (...) {
        // No C++ exception may cross the CXX boundary.
        return nullptr;
    }
}

} // namespace vvm::counter
