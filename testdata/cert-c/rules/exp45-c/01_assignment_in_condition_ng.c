/*
 * EXP45-C 違反例。
 * 選択文の制御式で代入を使用している。
 */

int check(int value)
{
    int status = 0;

    if (status = value) {
        return 1;
    }

    return 0;
}
