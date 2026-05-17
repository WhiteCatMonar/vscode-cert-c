/*
 * DCL37-C 違反例。
 * 標準ライブラリ関数を独自に定義している。
 */

void free(void *pointer)
{
    (void)pointer;
}
