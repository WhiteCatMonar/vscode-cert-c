/*
 * FIO41-C 違反例。
 * putcのストリーム引数に副作用を含めている。
 */

typedef struct file FILE;
extern int putc(int character, FILE *stream);

int output(FILE **stream)
{
    int character = 'A';
    return putc(character, *stream++);
}
