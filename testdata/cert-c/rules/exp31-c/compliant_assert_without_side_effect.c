/*
 * ルール: EXP31-C
 * 期待結果: 診断なし。
 * 参照元: https://www.jpcert.or.jp/sc-rules/
 */

#include <assert.h>

void validate(int index) {
  assert(index > 0);
  index++;
}
