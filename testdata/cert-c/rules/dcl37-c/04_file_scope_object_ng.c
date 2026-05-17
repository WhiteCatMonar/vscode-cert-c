/*
 * DCL37-C 違反例。
 * ファイルスコープのオブジェクトに予約済み識別子を使用している。
 */

typedef unsigned long size_t;

static const size_t _max_limit = 1024;
size_t _limit = 100;

int get_state(void)
{
    return (int)(_max_limit + _limit);
}
