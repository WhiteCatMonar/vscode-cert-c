/*
 * ENV33-C 違反例。
 * コマンドプロセッサが必要ない場面でsystem関数を呼び出している。
 */

int main(void)
{
    return system("date > /tmp/current-date.txt");
}
