/*
 * ENV33-C 違反例。
 * ENV33-C-EX1適用コメントにより情報レベルへ引き下げる。
 */

int main(void)
{
    /* cert-c: apply ENV33-C-EX1 */
    return system("command -v cc > /tmp/cc-path.txt");
}
