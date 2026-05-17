/*
 * INT33-C 違反例。
 * 0になり得る変数を剰余演算の右オペランドとして使用している。
 */

long remainder(long sl1, long sl2)
{
    return sl1 % sl2;
}
