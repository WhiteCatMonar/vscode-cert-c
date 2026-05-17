/*
 * MSC38-C 違反例。
 * assertマクロを関数ポインタとして渡そうとしている。
 */

#include <assert.h>

typedef void (*handler_type)(int);

void execute_handler(handler_type handler, int value)
{
    handler(value);
}

void validate(int value)
{
    execute_handler(assert, value);
}
