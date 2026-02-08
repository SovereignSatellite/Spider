# as f32.x64.s -o f32.x64.o
# objcopy -O binary -j .text f32.x64.o f32.x64.bin

.text
.intel_syntax noprefix

.align 32, 0xCC
square_root_f32:
    movd xmm0, edi
    sqrtss xmm0, xmm0
    movd eax, xmm0
    ret

.align 32, 0xCC
add_f32:
    movd xmm0, edi
    movd xmm1, esi
    addss xmm0, xmm1
    movd eax, xmm0
    ret

.align 32, 0xCC
subtract_f32:
    movd xmm0, edi
    movd xmm1, esi
    subss xmm0, xmm1
    movd eax, xmm0
    ret

.align 32, 0xCC
multiply_f32:
    movd xmm0, edi
    movd xmm1, esi
    mulss xmm0, xmm1
    movd eax, xmm0
    ret

.align 32, 0xCC
divide_f32:
    movd xmm0, edi
    movd xmm1, esi
    divss xmm0, xmm1
    movd eax, xmm0
    ret

.align 32, 0xCC
