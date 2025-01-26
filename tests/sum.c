#include <stdint.h>
#include <stdio.h>

extern int64_t sum(int64_t n);

int main() {
  for (int64_t n = 0; n < 10; ++n) {
    printf("%lld\n", sum(n));
  }
}
