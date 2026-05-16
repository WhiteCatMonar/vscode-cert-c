/*
 * ルール: PRE31-C
 * 期待結果: 診断なし。
 * 参照元: https://www.jpcert.or.jp/sc-rules/c-pre31-c.html
 */

#define LOCAL_ABS(x) (((x) < 0) ? -(x) : (x))

int normalize(int value) {
  value++;
  return LOCAL_ABS(value);
}

