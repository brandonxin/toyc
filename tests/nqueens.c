#include <stdint.h>
#include <stdio.h>

extern int64_t nqueens(int64_t n);

int main() {
  for (int64_t i = 1; i <= 12; ++i)
    printf("%lld\n", nqueens(i));
}
