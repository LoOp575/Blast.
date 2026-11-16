#include <stdint.h>
#include <stddef.h>

// Computes S(next) = S(current) + disk_total.
// Returns 1 on success, 0 for invalid output pointer or uint64_t overflow.
int growth_add(uint64_t current, uint64_t disk_total, uint64_t *result) {
    if (result == NULL) {
        return 0;
    }

    if (UINT64_MAX - current < disk_total) {
        return 0;
    }

    *result = current + disk_total;
    return 1;
}
