// RV32E-16B C runtime header.
// Include this in programs compiled for the 16-bit cell-addressed simulator.

#ifndef RV32E_RT_H
#define RV32E_RT_H

#define UART ((volatile char *)0x10000000)

static inline void putc(char c) {
    *UART = c;
}

static void puts(const char *s) {
    while (*s) {
        putc(*s);
        s++;
    }
}

// Binary long division by 10 (no hardware divide on RV32E).
static inline void _divmod10(unsigned n, unsigned *q, unsigned *r) {
    *q = 0;
    *r = 0;
    for (int i = 31; i >= 0; i--) {
        *r = (*r << 1) | ((n >> i) & 1);
        if (*r >= 10) {
            *r -= 10;
            *q |= 1u << i;
        }
    }
}

static void print_u32(unsigned n) {
    if (n >= 10) {
        unsigned q, r;
        _divmod10(n, &q, &r);
        print_u32(q);
        putc('0' + r);
    } else {
        putc('0' + n);
    }
}

static void print_i32(int n) {
    if (n < 0) {
        putc('-');
        print_u32((unsigned)(-(long long)n));
    } else {
        print_u32((unsigned)n);
    }
}

static inline void halt(void) {
    for (;;);
}

#endif
