#include <stdint.h>
#include <stdio.h>

extern int64_t gcd(int64_t a, int64_t b);

int main() {
  printf("%lld\n", gcd(0, 0));           // 0
  printf("%lld\n", gcd(17, 31));         // 1
  printf("%lld\n", gcd(37, 11));         // 1
  printf("%lld\n", gcd(10, 5));          // 5
  printf("%lld\n", gcd(54, 24));         // 6
  printf("%lld\n", gcd(123456, 789012)); // 12
  printf("%lld\n", gcd(0, 28));          // 28
  printf("%lld\n", gcd(42, 42));         // 42
}
