/*
 * DCL37-C 違反例。
 * 外部結合をもつ識別子として標準ライブラリ名を再定義している。
 */

void *malloc(unsigned long size)
{
    (void)size;
    return 0;
}
