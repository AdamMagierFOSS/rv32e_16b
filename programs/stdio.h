// stdio.h — Mock standard I/O for RV32E-16B simulator.
//
// Provides print functions through the UART MMIO at 0x10000000.
// Each character occupies one 16-bit cell (CHAR_BIT=16).
//
// No varargs (va_arg generates unsupported relocations on this target).
// Use the explicit print_* functions instead.

#ifndef _RV32E16B_STDIO_H
#define _RV32E16B_STDIO_H

#define EOF (-1)
#define NULL ((void *)0)

typedef unsigned int size_t;

#define UART ((volatile char *)0x10000000)

// --- Character I/O ---

static inline int putchar(int c) {
    *UART = (char)c;
    return c;
}

static inline int puts(const char *s) {
    while (*s)
        putchar(*s++);
    putchar('\n');
    return 0;
}

static inline void print_str(const char *s) {
    while (*s)
        putchar(*s++);
}

// --- Number formatting (no hardware divide on RV32E) ---

static void _divmod10(unsigned n, unsigned *q, unsigned *r) {
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

static void print_unsigned(unsigned n) {
    if (n >= 10) {
        unsigned q, r;
        _divmod10(n, &q, &r);
        print_unsigned(q);
        putchar('0' + r);
    } else {
        putchar('0' + n);
    }
}

static void print_int(int n) {
    if (n < 0) {
        putchar('-');
        print_unsigned((unsigned)(-(long long)n));
    } else {
        print_unsigned((unsigned)n);
    }
}

static void print_hex(unsigned n) {
    print_str("0x");
    if (n == 0) {
        putchar('0');
        return;
    }
    int started = 0;
    for (int shift = 28; shift >= 0; shift -= 4) {
        unsigned nibble = (n >> shift) & 0xf;
        if (nibble || started) {
            putchar(nibble < 10 ? '0' + nibble : 'a' + nibble - 10);
            started = 1;
        }
    }
}

// --- Memory operations ---

static void *memset(void *s, int c, size_t n) {
    char *p = (char *)s;
    for (size_t i = 0; i < n; i++)
        p[i] = (char)c;
    return s;
}

static void *memcpy(void *dest, const void *src, size_t n) {
    char *d = (char *)dest;
    const char *s = (const char *)src;
    for (size_t i = 0; i < n; i++)
        d[i] = s[i];
    return dest;
}

static size_t strlen(const char *s) {
    size_t n = 0;
    while (*s++) n++;
    return n;
}

static int strcmp(const char *a, const char *b) {
    while (*a && *a == *b) { a++; b++; }
    return *a - *b;
}

#endif // _RV32E16B_STDIO_H
