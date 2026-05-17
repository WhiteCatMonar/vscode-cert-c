//! MSC38-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::Token;
use crate::rules::common::is_declaration_keyword;

/// MSC38-Cの診断を返す。
///
/// 定義済みマクロ名をマクロ定義または宣言で再定義している疑いがある場合に診断する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// MSC38-Cに関する診断一覧。
///
/// TODO: C99で定義されるすべての定義済みマクロと処理系拡張を環境モデルへ移す。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for window in tokens.windows(3) {
        if window[0].text == "#"
            && window[1].text == "define"
            && is_macro_object_identifier(&window[2].text)
        {
            diagnostics.push(new_diagnostic(source, &window[2]));
        }
    }

    for window in tokens.windows(2) {
        if is_declaration_keyword(&window[0].text) && is_predefined_macro(&window[1].text) {
            diagnostics.push(new_diagnostic(source, &window[1]));
        }
    }

    for (index, token) in tokens.iter().enumerate() {
        if is_standard_library_function_like_macro(&token.text)
            && is_object_like_macro_use(tokens, index)
        {
            diagnostics.push(new_diagnostic(source, token));
        }
    }

    diagnostics
}

/// MSC38-Cの診断を作成する。
///
/// # 引数
///
/// - `source`: 診断位置を行番号・列番号へ変換するための元ソースコード。
/// - `token`: 診断対象のトークン。
///
/// # 戻り値
///
/// MSC38-C診断。
fn new_diagnostic(source: &str, token: &Token) -> Diagnostic {
    Diagnostic::new(
        source,
        token.span,
        "MSC38-C",
        "マクロとして実装されている可能性のある定義済みの識別子をオブジェクトとして扱わない",
    )
}

/// 定義済みマクロとして扱う識別子かを返す。
///
/// # 引数
///
/// - `value`: 判定対象の識別子名。
///
/// # 戻り値
///
/// 定義済みマクロとして扱う場合は`true`。
fn is_predefined_macro(value: &str) -> bool {
    matches!(
        value,
        "__DATE__"
            | "__FILE__"
            | "__LINE__"
            | "__STDC__"
            | "__STDC_HOSTED__"
            | "__STDC_VERSION__"
            | "__TIME__"
    )
}

/// マクロ定義でオブジェクトとして扱ってはいけない識別子かを返す。
///
/// # 引数
///
/// - `value`: 判定対象の識別子名。
///
/// # 戻り値
///
/// マクロ定義で検出対象として扱う場合は`true`。
fn is_macro_object_identifier(value: &str) -> bool {
    value == "errno" || is_standard_library_function_like_macro(value) || is_predefined_macro(value)
}

/// 標準ライブラリで関数形式マクロとして実装される可能性がある識別子かを返す。
///
/// # 引数
///
/// - `value`: 判定対象の識別子名。
///
/// # 戻り値
///
/// 関数形式マクロとして扱う場合は`true`。
///
/// TODO: C99の標準ライブラリマクロ一覧を環境モデルへ移す。
fn is_standard_library_function_like_macro(value: &str) -> bool {
    matches!(
        value,
        "assert" | "setjmp" | "va_arg" | "va_copy" | "va_end" | "va_start"
    )
}

/// 関数形式マクロ名をオブジェクトのように使用しているかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象のマクロ名トークン位置。
///
/// # 戻り値
///
/// 直後がマクロ呼び出しの`(`ではなく、宣言やマクロ定義でもない場合は`true`。
fn is_object_like_macro_use(tokens: &[Token], index: usize) -> bool {
    !is_macro_definition_name(tokens, index)
        && !is_declared_macro_name(tokens, index)
        && !is_include_header_name(tokens, index)
        && next_non_space(tokens, index).is_none_or(|next_index| tokens[next_index].text != "(")
}

/// include指令のヘッダ名として現れているかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象の識別子トークン位置。
///
/// # 戻り値
///
/// `#include <name.h>`の`name`部分として扱う場合は`true`。
fn is_include_header_name(tokens: &[Token], index: usize) -> bool {
    index >= 3
        && tokens[index - 3].text == "#"
        && tokens[index - 2].text == "include"
        && tokens[index - 1].text == "<"
}

/// マクロ定義名として現れているかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象の識別子トークン位置。
///
/// # 戻り値
///
/// `#define`直後の名前の場合は`true`。
fn is_macro_definition_name(tokens: &[Token], index: usize) -> bool {
    index >= 2 && tokens[index - 2].text == "#" && tokens[index - 1].text == "define"
}

/// 宣言名として現れているかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象の識別子トークン位置。
///
/// # 戻り値
///
/// 宣言キーワード直後の場合は`true`。
fn is_declared_macro_name(tokens: &[Token], index: usize) -> bool {
    index > 0 && is_declaration_keyword(&tokens[index - 1].text)
}

/// 空白を除いた次のトークン位置を返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 検索開始位置の直前。
///
/// # 戻り値
///
/// 空白以外の次トークン位置。存在しない場合は`None`。
fn next_non_space(tokens: &[Token], index: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(index + 1)
        .find_map(|(next_index, token)| (token.text != " ").then_some(next_index))
}
