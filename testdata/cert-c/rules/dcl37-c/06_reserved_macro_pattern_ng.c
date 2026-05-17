/*
 * DCL37-C 違反例。
 * 予約済みマクロ名や予約済み識別子パターンを使用している。
 */

typedef short int_fast16_t;

#define SIZE_MAX 80

enum { SIZE_MAX = 80 };
static const int_fast16_t INTFAST16_LIMIT_MAX = 12000;

int get_limit(void)
{
    return SIZE_MAX + (int)INTFAST16_LIMIT_MAX;
}
