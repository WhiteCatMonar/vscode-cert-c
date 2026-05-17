/*
 * FIO41-C 適合例。
 * putcの引数に副作用を含めない。
 */

typedef struct file FILE;
extern int putc(int character, FILE *stream);

int output(FILE *stream)
{
    int character = 'A';
    return putc(character, stream);
}
