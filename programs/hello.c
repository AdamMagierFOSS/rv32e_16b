#include "stdio.h"

void main(void) {
    puts("Hello from RV32E-16B!");
    print_str("CHAR_BIT = "); print_int(__CHAR_BIT__); putchar('\n');
    print_str("sizeof(char)  = "); print_unsigned(sizeof(char)); putchar('\n');
    print_str("sizeof(short) = "); print_unsigned(sizeof(short)); putchar('\n');
    print_str("sizeof(int)   = "); print_unsigned(sizeof(int)); putchar('\n');
    print_str("sizeof(void*) = "); print_unsigned(sizeof(void *)); putchar('\n');
    print_str("hex: "); print_hex(0xDEADBEEF); putchar('\n');
    print_str("negative: "); print_int(-42); putchar('\n');
    print_str("strlen(\"Hi\") = "); print_unsigned(strlen("Hi")); putchar('\n');
}
