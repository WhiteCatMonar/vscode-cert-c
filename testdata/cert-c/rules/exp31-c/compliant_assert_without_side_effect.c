/*
 * ルール: EXP31-C
 * 期待結果: 診断なし。
 */

#include <assert.h>

void validate(int index) {
  assert(index > 0);
  index++;
}
