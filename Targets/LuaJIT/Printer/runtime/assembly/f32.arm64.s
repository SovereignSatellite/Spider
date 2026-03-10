// as f32.arm64.s -o f32.arm64.o
// segedit f32.arm64.o -extract __TEXT __text f32.arm64.bin

.text

.p2align 5
square_root_f32:
    fmov s0, w0
    fsqrt s0, s0
    fmov w0, s0
    ret

.p2align 5
add_f32:
    fmov s0, w0
    fmov s1, w1
    fadd s0, s0, s1
    fmov w0, s0
    ret

.p2align 5
subtract_f32:
    fmov s0, w0
    fmov s1, w1
    fsub s0, s0, s1
    fmov w0, s0
    ret

.p2align 5
multiply_f32:
    fmov s0, w0
    fmov s1, w1
    fmul s0, s0, s1
    fmov w0, s0
    ret

.p2align 5
divide_f32:
    fmov s0, w0
    fmov s1, w1
    fdiv s0, s0, s1
    fmov w0, s0
    ret

.p2align 5
