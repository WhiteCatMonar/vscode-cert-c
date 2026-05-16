/*
 * ルール: DCL37-C
 * 期待結果: 診断なし。
 * 参照元: https://www.jpcert.or.jp/sc-rules/
 */

int internal_state;

int main(void) {
  return internal_state;
}

