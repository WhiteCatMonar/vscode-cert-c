/*
 * EXP44-C 適合例。
 * sizeofのオペランドに副作用を含めない。
 */

int main(void)
{
    int value = 1;
    return (int)sizeof(value);
}
