#include <stdio.h>
#include "hello.h"

void hello() {
#ifdef IS_RELEASE
  printf("hello release\n");
#else
  printf("hello debug\n");
#endif
}

int32_t returns42() {
  return 42;
}
