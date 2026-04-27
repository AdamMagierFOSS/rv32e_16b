.section .text._start
.global _start
_start:
    lui sp, %hi(__stack_top)
    addi sp, sp, %lo(__stack_top)
    jal ra, main
    ebreak
