/*
 * PRE32-C 適合例。
 * 前処理指令の分岐ごとに関数呼び出しを完結させている。
 */

#include <string.h>

void func(const char *src)
{
    char *dest;

#ifdef PLATFORM1
    memcpy(dest, src, 12);
#else
    memcpy(dest, src, 24);
#endif
}
