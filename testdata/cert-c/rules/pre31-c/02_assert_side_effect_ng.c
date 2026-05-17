/*
 * PRE31-C 違反例。
 * assert引数に副作用を含めている。
 */

#include <assert.h>

void validate(int index)
{
    assert(index++ > 0);
}

