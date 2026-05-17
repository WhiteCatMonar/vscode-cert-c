/*
 * MSC30-C 違反例。
 * rand関数を使用している。
 */

extern int rand(void);

int main(void)
{
    return rand();
}
