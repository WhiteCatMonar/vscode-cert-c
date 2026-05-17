/*
 * DCL37-C 違反例。
 * 予約済み識別子を宣言している。
 */

int _internal_state;

int main(void) {
  return _internal_state;
}

