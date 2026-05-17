/*
 * ルール: EXP31-C
 * 期待結果: assert引数内の副作用として診断する。
 */

#include <assert.h>

void validate(int index) {
  assert(index++ > 0);
}

