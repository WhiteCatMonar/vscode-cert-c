/*
 * INT33-C 違反例。
 * 0になり得る変数を除算の右オペランドとして使用している。
 */

long divide(long sl1, long sl2)
{
    return sl1 / sl2;
}
