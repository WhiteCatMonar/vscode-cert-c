/*
 * DCL37-C 適合例。
 * ファイルスコープのオブジェクトに予約済み識別子を使用しない。
 */

typedef unsigned long size_t;

static const size_t max_limit = 1024;
size_t limit = 100;

int get_state(void)
{
    return (int)(max_limit + limit);
}
