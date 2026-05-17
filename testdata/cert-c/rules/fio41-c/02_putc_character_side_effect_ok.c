/*
 * FIO41-C 適合例。
 * putcの文字引数だけに副作用を含めている。
 */

typedef struct file FILE;
extern int putc(int character, FILE *stream);

int output(FILE *stream)
{
    int character = 'A';
    return putc(character++, stream);
}
