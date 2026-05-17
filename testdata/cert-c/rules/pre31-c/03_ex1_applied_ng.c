/*
 * PRE31-C 違反例。
 * PRE31-C-EX1適用コメントにより情報レベルへ引き下げる。
 */

#include <assert.h>

int read_value(int *value)
{
    /* cert-c: apply PRE31-C-EX1 */
    assert((*value)++ > 0);
    return *value;
}
