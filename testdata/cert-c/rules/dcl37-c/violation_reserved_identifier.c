/*
 * ルール: DCL37-C
 * 期待結果: 予約済み識別子の宣言として診断する。
 */

int _internal_state;

int main(void) {
  return _internal_state;
}

