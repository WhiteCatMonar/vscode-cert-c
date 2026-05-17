//! POS33-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::Token;
use crate::rules::common::{find_call_arguments, is_declared_name};

/// POS33-Cの診断を返す。
///
/// `vfork()`呼び出しを検出する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// POS33-Cに関する診断一覧。
///
/// TODO: POSIXルールの有効化設定を追加してから既定有効範囲を調整する。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if token.text != "vfork"
            || is_declared_name(tokens, index)
            || find_call_arguments(tokens, index).is_none()
        {
            continue;
        }

        diagnostics.push(Diagnostic::new(
            source,
            token.span,
            "POS33-C",
            "vfork() を使用しない",
        ));
    }

    diagnostics
}
