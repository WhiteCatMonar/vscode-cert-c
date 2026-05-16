//! INT33-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::Token;

/// INT33-Cの診断を返す。
///
/// 除算または剰余演算の右辺が定数0リテラルである場合に診断する。
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

    for window in tokens.windows(2) {
        if matches!(window[0].text.as_str(), "/" | "%") && is_zero_literal(&window[1].text) {
            diagnostics.push(Diagnostic::new(
                source,
                window[1].span,
                "INT33-C",
                "除算または剰余演算の右辺が0にならないことを保証してください。",
            ));
        }
    }

    diagnostics
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
