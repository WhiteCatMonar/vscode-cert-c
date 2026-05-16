//! DCL37-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::Token;

/// DCL37-Cの診断を返す。
///
/// 宣言キーワード直後の識別子が予約済み識別子に見える場合に診断する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// DCL37-Cに関する診断一覧。
///
/// TODO: スコープとC標準ヘッダ由来の識別子を考慮する。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let declaration_keywords = [
        "char", "double", "enum", "float", "int", "long", "short", "signed", "struct", "union",
        "unsigned", "void",
    ];

    for window in tokens.windows(2) {
        if declaration_keywords.contains(&window[0].text.as_str())
            && is_reserved_identifier(&window[1].text)
        {
            diagnostics.push(Diagnostic::new(
                source,
                window[1].span,
                "DCL37-C",
                "予約済み識別子を宣言または定義しないでください。",
            ));
        }
    }

    diagnostics
}

/// 予約済み識別子に該当する可能性がある名前かを判定する。
///
/// # 引数
///
/// - `value`: 判定対象の識別子名。
///
/// # 戻り値
///
/// 予約済み識別子として扱う場合は`true`。
///
/// TODO: C99の予約済み識別子規則と処理系予約領域を正確に反映する。
fn is_reserved_identifier(value: &str) -> bool {
    value.starts_with("__") || value.starts_with('_')
}
