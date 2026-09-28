// Extra definitions for the cbindgen generated headers.
// Declarations are written in cbindgen.toml

// Finally this is impossible, it changes the C signature from `struct` to a C++ `class` which won't link with the produced library.
#ifdef TESTING_SOMETHING_IMPOSSIBLE

#include "kurbo-ffi.h"

// kurboPos & glm::vec2 adapters
kurboPos::kurboPos(const glm::vec2& value)
    : x(value.x), y(value.y) {}

kurboPos& kurboPos::operator=(const glm::vec2& value) {
    x = value.x;
    y = value.y;
    return *this;
}

kurboPos::operator glm::vec2() const {
    return glm::vec2{x, y};
}

// Some sanity checks for dirty kurboPos modifications
#ifdef DEBUG
#include <cstddef>
#include <type_traits>
static_assert(sizeof(kurboPos) == sizeof(double) * 2);
static_assert(offsetof(kurboPos, x) == 0);
static_assert(offsetof(kurboPos, y) == sizeof(double));
static_assert(std::is_standard_layout_v<kurboPos>);
#endif // DEBUG

#endif // TESTING_SOMETHING_IMPOSSIBLE
