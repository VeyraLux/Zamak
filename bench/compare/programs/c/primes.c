#include <stdio.h>
#include <stdint.h>

int main(void) {
    int count = 0;
    for (int64_t n = 2; n < 1000000; n++) {
        int prime = 1;
        for (int64_t d = 2; d * d <= n; d++) {
            if (n % d == 0) { prime = 0; break; }
        }
        if (prime) count++;
    }
    printf("%d\n", count);
    return 0;
}
