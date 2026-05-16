/*
 * ルール: INT33-C
 * 期待結果: 定数0による除算として診断する。
 * 参照元: https://www.jpcert.or.jp/sc-rules/
 */

int divide(int value) {
  return value / 0;
}

