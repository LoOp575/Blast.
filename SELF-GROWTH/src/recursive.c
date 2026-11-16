#include <stdint.h>
#include <limits.h>

// x(n+1) = x(n) * x(n) + 1
// Returns 1 on success, 0 if the result would overflow uint64_t.
int recursive_step(uint64_t x, uint64_t *result) {
    if (result == 0) {
        return 0;
    }

    if (x != 0 && x > (UINT64_MAX - 1) / x) {
        return 0;
    }

    *result = x * x + 1;
    return 1;
}
