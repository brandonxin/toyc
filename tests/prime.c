#include <stdint.h>
#include <stdio.h>

extern int64_t prime(int64_t x);

int main() {
  for (int64_t i = 0; i < 100; ++i) {
    if (prime(i)) {
      printf("%lld\n", i);
    }
  }
}
