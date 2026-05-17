//! MSC30-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::Token;
use crate::rules::common::{find_call_arguments, is_declared_name};

/// MSC30-Cの診断を返す。
///
/// `rand()`呼び出しを検出する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// MSC30-Cに関する診断一覧。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if token.text != "rand"
            || is_declared_name(tokens, index)
            || find_call_arguments(tokens, index).is_none()
        {
            continue;
        }

        diagnostics.push(Diagnostic::new(
            source,
            token.span,
            "MSC30-C",
            "疑似乱数の生成に rand() 関数を使用しない",
        ));
    }

    diagnostics
}
