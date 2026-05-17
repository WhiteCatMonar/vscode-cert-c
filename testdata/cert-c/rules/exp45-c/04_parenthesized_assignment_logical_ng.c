/*
 * EXP45-C 違反例。
 * 括弧で囲んだ代入式の結果を論理演算の左オペランドとして使用している。
 */

int check(int v, int w, int flag)
{
    if ((v = w) && flag) {
        return 1;
    }

    return 0;
}
