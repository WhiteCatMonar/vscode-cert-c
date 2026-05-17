//! FIO41-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::Token;
use crate::rules::common::{find_call_arguments, find_side_effect};

/// FIO41-Cの診断を返す。
///
/// `getc`、`putc`、`getwc`、`putwc`のストリーム引数に副作用が含まれる場合に診断する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// FIO41-Cに関する診断一覧。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let target_names = ["getc", "putc", "getwc", "putwc"];

    for (index, token) in tokens.iter().enumerate() {
        if !target_names.contains(&token.text.as_str()) {
            continue;
        }

        let Some(call) = find_call_arguments(tokens, index) else {
            continue;
        };

        let Some(stream_argument) = stream_argument(&token.text, &call.arguments) else {
            continue;
        };

        if let Some(side_effect) = find_side_effect(stream_argument) {
            diagnostics.push(Diagnostic::new(
                source,
                side_effect.span,
                "FIO41-C",
                "副作用を持つストリーム引数を getc()、putc()、getwc()、putwc() に渡さない",
            ));
        }
    }

    diagnostics
}

/// 対象関数のストリーム引数トークン列を返す。
///
/// # 引数
///
/// - `function_name`: 呼び出し対象の関数名または関数形式マクロ名。
/// - `arguments`: 呼び出し実引数のトークン列。
///
/// # 戻り値
///
/// ストリーム引数に該当する実引数。引数が不足している場合は`None`。
fn stream_argument<'a>(function_name: &str, arguments: &'a [Vec<Token>]) -> Option<&'a [Token]> {
    let argument_index = match function_name {
        "getc" | "getwc" => 0,
        "putc" | "putwc" => 1,
        _ => return None,
    };

    arguments.get(argument_index).map(Vec::as_slice)
}
