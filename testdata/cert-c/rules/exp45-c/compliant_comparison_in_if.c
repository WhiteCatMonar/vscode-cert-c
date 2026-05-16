/*
 * ルール: EXP45-C
 * 期待結果: 診断なし。
 * 参照元: https://www.jpcert.or.jp/sc-rules/
 */

int check(int value) {
  int status = 0;

  if (status == value) {
    return 1;
  }

  return 0;
}

