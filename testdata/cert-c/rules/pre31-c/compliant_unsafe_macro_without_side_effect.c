/*
 * ルール: PRE31-C
 * 期待結果: 診断なし。
 */

#define LOCAL_ABS(x) (((x) < 0) ? -(x) : (x))

int normalize(int value) {
  value++;
  return LOCAL_ABS(value);
}

