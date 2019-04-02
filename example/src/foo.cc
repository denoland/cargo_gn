#include "hello.h"

int main() {
  if (returns42() != 42) {
    return 1;
  }
  hello();
  return 0;
}
