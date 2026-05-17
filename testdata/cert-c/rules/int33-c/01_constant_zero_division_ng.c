/*
 * INT33-C 違反例。
 * 定数0で除算している。
 */

int divide(int value) {
  return value / 0;
}

