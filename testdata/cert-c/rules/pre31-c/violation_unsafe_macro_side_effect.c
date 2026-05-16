/*
 * ルール: PRE31-C
 * 期待結果: 安全でない関数形式マクロへの副作用を持つ引数として診断する。
 * 参照元: https://www.jpcert.or.jp/sc-rules/c-pre31-c.html
 */

#define LOCAL_ABS(x) (((x) < 0) ? -(x) : (x))

int normalize(int value) {
  return LOCAL_ABS(++value);
}

