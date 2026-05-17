/*
 * PRE31-C 適合例。
 * assert引数に副作用を含めない。
 */

#include <assert.h>

void validate(int index)
{
    assert(index > 0);
    index++;
}
