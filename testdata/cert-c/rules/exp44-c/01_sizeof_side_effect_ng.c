/*
 * EXP44-C 違反例。
 * sizeofのオペランドに副作用を含めている。
 */

int main(void)
{
    int value = 1;
    return (int)sizeof(value++);
}
