/*
 * MSC38-C 違反例。
 * errnoを自前でマクロ定義している。
 */

#define errno 1

int main(void)
{
    return errno;
}
