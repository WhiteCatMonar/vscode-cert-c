/*
 * MSC38-C 適合例。
 * assertマクロを通常の関数形式マクロ呼び出しとして使用している。
 */

#include <assert.h>

void validate(int value)
{
    assert(value != 0);
}
