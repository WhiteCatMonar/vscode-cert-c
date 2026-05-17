/*
 * POS33-C 違反例。
 * vfork関数を使用している。
 */

extern int vfork(void);

int main(void)
{
    return vfork();
}
