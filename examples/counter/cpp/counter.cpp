#include "counter.hpp"

#include "Vcounter.h"
#include "verilated.h"

#include <cstdint>
#include <memory>

namespace vvm::counter {

class Counter::Impl final {
public:
    Impl()
        : context{std::make_unique<VerilatedContext>()},
          model{std::make_unique<Vcounter>(context.get())} {
        CData low{0};

        model->clk(low);
        model->reset_n(low);
        model->enable(low);
    }

    std::unique_ptr<VerilatedContext> context;
    std::unique_ptr<Vcounter> model;
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

    impl_->finished = true;
    impl_->model->final();
}

void Counter::set_clk(const bool value) noexcept {
    CData raw_value = static_cast<CData>(value);
    impl_->model->clk(raw_value);
}

void Counter::set_reset_n(const bool value) noexcept {
    CData raw_value = static_cast<CData>(value);
    impl_->model->reset_n(raw_value);
}

void Counter::set_enable(const bool value) noexcept {
    CData raw_value = static_cast<CData>(value);
    impl_->model->enable(raw_value);
}

std::uint8_t Counter::count() const noexcept {
    return static_cast<std::uint8_t>(impl_->model->count());
}

std::unique_ptr<Counter> create_counter() noexcept {
    try {
        return std::make_unique<Counter>();
    } catch (...) {
        return nullptr;
    }
}

} // namespace vvm::counter
