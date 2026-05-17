/*
 * ルール: INT33-C
 * 期待結果: 定数0による除算として診断する。
 */

int divide(int value) {
  return value / 0;
}

