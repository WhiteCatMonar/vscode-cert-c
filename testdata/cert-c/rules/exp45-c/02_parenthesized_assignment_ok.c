/*
 * EXP45-C 適合例。
 * 代入式を括弧で明示したうえで比較している。
 */

int check(int value)
{
    int status = 0;

    if ((status = value) != 0) {
        return 1;
    }

    return 0;
}
