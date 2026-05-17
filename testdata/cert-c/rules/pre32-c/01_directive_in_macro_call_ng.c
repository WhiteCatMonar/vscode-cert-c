#include <string.h>

void func(const char *src) {
  /* ソース文字列を検証しサイズを計算 */
  char *dest;
  /* コピー先文字列のために malloc() */
  memcpy(dest, src,
    #ifdef PLATFORM1
      12
    #else
      24
    #endif
  );
  /* ... */
}