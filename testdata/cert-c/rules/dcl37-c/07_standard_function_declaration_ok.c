/*
 * DCL37-C 適合例。
 * 標準ライブラリ関数と適合するプロトタイプ宣言を行っている。
 */

void free(void *);

void release_memory(void *pointer)
{
    free(pointer);
}
