//! INT33-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::Token;

/// INT33-Cの診断を返す。
///
/// 除算または剰余演算の右辺が定数0または0になり得る識別子である場合に診断する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// INT33-Cに関する診断一覧。
///
/// TODO: 変数が0になり得るかをデータフロー解析で判定する。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if !matches!(token.text.as_str(), "/" | "%") {
            continue;
        }

        let Some(divisor) = tokens.get(index + 1) else {
            continue;
        };

        if is_potentially_zero_divisor(&divisor.text) {
            diagnostics.push(Diagnostic::new(
                source,
                divisor.span,
                "INT33-C",
                "除算および剰余演算がゼロ除算エラーを引き起こさないことを保証する",
            ));
        }
    }

    diagnostics
}

/// 0になり得る除数として扱うトークンかを返す。
///
/// # 引数
///
/// - `value`: 判定対象のトークン文字列。
///
/// # 戻り値
///
/// 定数0または識別子の場合は`true`。
///
/// TODO: データフロー解析で、除数が0にならないことを確認済みの経路を除外する。
fn is_potentially_zero_divisor(value: &str) -> bool {
    is_zero_literal(value) || is_identifier(value)
}

/// 整数定数0として扱うリテラルかを返す。
///
/// # 引数
///
/// - `value`: 判定対象のトークン文字列。
///
/// # 戻り値
///
/// 整数定数0として扱う場合は`true`。
fn is_zero_literal(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    matches!(lower.as_str(), "0" | "0u" | "0l" | "0ul" | "0lu")
}

/// C識別子として扱える簡易トークンかを返す。
///
/// # 引数
///
/// - `value`: 判定対象のトークン文字列。
///
/// # 戻り値
///
/// ASCII識別子の場合は`true`。
fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
}
