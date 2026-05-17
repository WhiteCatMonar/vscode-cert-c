/*
 * DCL37-C 適合例。
 * 予約済みマクロ名や予約済み識別子パターンを使用しない。
 */

typedef short int_fast16_t;

enum { BUFFER_SIZE = 80 };
static const int_fast16_t my_intfast16_upper_limit = 12000;

int get_limit(void)
{
    return BUFFER_SIZE + (int)my_intfast16_upper_limit;
}
