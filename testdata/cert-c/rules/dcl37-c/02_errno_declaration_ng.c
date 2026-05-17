/*
 * DCL37-C 違反例。
 * errnoを自前で宣言している。
 */

extern int errno;

int main(void)
{
    return errno;
}
