/*
 * PRE31-C 違反例。
 * 安全でない関数形式マクロへ副作用を持つ引数を渡している。
 */

#define LOCAL_ABS(x) (((x) < 0) ? -(x) : (x))

int normalize(int value)
{
    return LOCAL_ABS(value++);
}
