/*
 * ルール: DCL37-C
 * 期待結果: 予約済み識別子の宣言として診断する。
 * 参照元: https://www.jpcert.or.jp/sc-rules/
 */

int _internal_state;

int main(void) {
  return _internal_state;
}

