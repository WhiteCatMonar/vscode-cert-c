/*
 * ルール: DCL37-C
 * 期待結果: 診断なし。
 */

int internal_state;

int main(void) {
  return internal_state;
}

