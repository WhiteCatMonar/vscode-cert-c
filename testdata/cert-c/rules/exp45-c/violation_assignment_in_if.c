/*
 * ルール: EXP45-C
 * 期待結果: 制御式で使われた代入として診断する。
 */

int check(int value) {
  int status = 0;

  if (status = value) {
    return 1;
  }

  return 0;
}

