#include <stdint.h>
#include <stdio.h>

extern int64_t fib(int64_t x);

int main() {
  for (int64_t i = 0; i < 10; ++i)
    printf("%lld\n", fib(i));
  return 0;
}
