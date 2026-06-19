#include "counter.hpp"

#include <cstdint>

std::uint8_t count_width() {
    return 8;
}

bool reset_is_active_low() {
    return true;
}
