/*
 * PRE31-C 適合例。
 * 安全でない関数形式マクロへ副作用を持たない引数を渡している。
 */

#define LOCAL_ABS(x) (((x) < 0) ? -(x) : (x))

int normalize(int value) {
  value++;
  return LOCAL_ABS(value);
}

