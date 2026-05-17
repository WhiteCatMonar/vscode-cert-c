/*
 * DCL37-C 適合例。
 * 外部結合をもつ識別子として標準ライブラリ名を再定義しない。
 */

void *my_malloc(unsigned long size)
{
    (void)size;
    return 0;
}
