/*
 * MSC38-C 違反例。
 * 定義済み識別子をオブジェクトとして再定義している。
 */

#define __LINE__ 100

int line_value(void)
{
    return __LINE__;
}
