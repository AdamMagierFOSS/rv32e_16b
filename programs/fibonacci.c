#include "stdio.h"

volatile unsigned *input  = (volatile unsigned *)0x1000;
volatile unsigned *output = (volatile unsigned *)0x1002;

unsigned fibonacci(unsigned n) {
    if (n <= 1) return n;
    unsigned a = 0, b = 1;
    for (unsigned i = 2; i <= n; i++) {
        unsigned tmp = a + b;
        a = b;
        b = tmp;
    }
    return b;
}

void main(void) {
    unsigned n = *input;
    unsigned result = fibonacci(n);
    *output = result;
    print_str("fib("); print_unsigned(n); print_str(") = "); print_unsigned(result); putchar('\n');
}
