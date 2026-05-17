/*
 * EXP45-C 適合例。
 * do-whileの制御式で、代入式をコンマ式の途中要素として使用している。
 */

int check(int x, int y, int p, int q)
{
    do {
        x++;
    } while (x = y, p == q);

    return x;
}
