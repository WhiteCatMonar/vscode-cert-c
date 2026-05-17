//! DCL37-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::Token;
use crate::rules::common::is_declaration_keyword;

/// DCL37-Cの診断を返す。
///
/// 宣言やマクロ定義に予約済み識別子が現れる場合に診断する。
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

    for (index, token) in tokens.iter().enumerate() {
        if is_reserved_macro_definition(tokens, index)
            || is_reserved_declaration_name(tokens, index)
            || is_reserved_external_identifier(tokens, index)
        {
            diagnostics.push(new_diagnostic(source, token));
        }
    }

    diagnostics
}

/// DCL37-Cの診断を作成する。
///
/// # 引数
///
/// - `source`: 診断位置を行番号・列番号へ変換するための元ソースコード。
/// - `token`: 診断対象のトークン。
///
/// # 戻り値
///
/// DCL37-C診断。
fn new_diagnostic(source: &str, token: &Token) -> Diagnostic {
    Diagnostic::new(
        source,
        token.span,
        "DCL37-C",
        "予約済み識別子の宣言や定義をしない",
    )
}

/// 予約済み識別子をマクロ定義名として扱っているかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象のトークン位置。
///
/// # 戻り値
///
/// 予約済みマクロ名として扱う場合は`true`。
fn is_reserved_macro_definition(tokens: &[Token], index: usize) -> bool {
    index >= 2
        && tokens[index - 2].text == "#"
        && tokens[index - 1].text == "define"
        && is_reserved_identifier(&tokens[index].text)
}

/// 予約済み識別子を宣言名として扱っているかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象のトークン位置。
///
/// # 戻り値
///
/// 宣言名として扱う場合は`true`。
fn is_reserved_declaration_name(tokens: &[Token], index: usize) -> bool {
    is_reserved_identifier(&tokens[index].text)
        && (is_previous_declaration_keyword(tokens, index)
            || is_pointer_declaration_name(tokens, index))
        || is_reserved_file_scope_object(tokens, index)
        || is_reserved_enum_constant(tokens, index)
}

/// 直前に宣言キーワードがあるかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象のトークン位置。
///
/// # 戻り値
///
/// 宣言キーワード直後の場合は`true`。
fn is_previous_declaration_keyword(tokens: &[Token], index: usize) -> bool {
    index > 0 && is_declaration_keyword(&tokens[index - 1].text)
}

/// ポインタ宣言の名前として現れているかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象のトークン位置。
///
/// # 戻り値
///
/// 宣言キーワードと`*`に続く名前の場合は`true`。
fn is_pointer_declaration_name(tokens: &[Token], index: usize) -> bool {
    index >= 2 && tokens[index - 1].text == "*" && is_declaration_keyword(&tokens[index - 2].text)
}

/// ファイルスコープオブジェクト名として予約済み識別子が現れているかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象のトークン位置。
///
/// # 戻り値
///
/// ファイルスコープ宣言のオブジェクト名として扱う場合は`true`。
///
/// TODO: 宣言Parserとスコープ情報でファイルスコープを正確に判定する。
fn is_reserved_file_scope_object(tokens: &[Token], index: usize) -> bool {
    is_reserved_identifier(&tokens[index].text)
        && is_probable_declaration_name(tokens, index)
        && !tokens.get(index + 1).is_some_and(|token| token.text == "(")
}

/// 予約済み列挙定数名として現れているかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象のトークン位置。
///
/// # 戻り値
///
/// enum定義内の予約済み定数名として扱う場合は`true`。
///
/// TODO: enum宣言Parserで範囲と列挙子を正確に判定する。
fn is_reserved_enum_constant(tokens: &[Token], index: usize) -> bool {
    is_reserved_identifier(&tokens[index].text)
        && index > 0
        && matches!(tokens[index - 1].text.as_str(), "{" | ",")
        && has_previous_token_before_block(tokens, index, "enum")
        && tokens
            .get(index + 1)
            .is_some_and(|token| matches!(token.text.as_str(), "=" | "," | "}"))
}

/// 簡易的に宣言名らしい位置かを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象のトークン位置。
///
/// # 戻り値
///
/// 直近の文境界以降に型名または型修飾子があり、直後が宣言継続トークンの場合は`true`。
fn is_probable_declaration_name(tokens: &[Token], index: usize) -> bool {
    if index == 0 {
        return false;
    }

    let start = previous_statement_boundary(tokens, index);
    let before = &tokens[start..index];
    let has_type_context = before.iter().any(|token| {
        is_declaration_keyword(&token.text)
            || is_type_qualifier(&token.text)
            || is_known_type_name(&token.text)
    });
    let has_statement_token = before.iter().any(|token| {
        matches!(
            token.text.as_str(),
            "=" | "return" | "(" | ")" | "+" | "-" | "," | ";"
        )
    });
    let has_declarator_follower = tokens
        .get(index + 1)
        .is_some_and(|token| matches!(token.text.as_str(), ";" | "=" | "," | "[" | ")"));

    has_type_context && !has_statement_token && has_declarator_follower
}

/// 直近の文境界の次の位置を返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象のトークン位置。
///
/// # 戻り値
///
/// 直近の`;`、`{`、`}`の次のトークン位置。
fn previous_statement_boundary(tokens: &[Token], index: usize) -> usize {
    tokens[..index]
        .iter()
        .rposition(|token| matches!(token.text.as_str(), ";" | "{" | "}"))
        .map_or(0, |position| position + 1)
}

/// 直近のブロックまたは文境界以降に指定トークンがあるかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 検索終端位置。
/// - `text`: 検索するトークン文字列。
///
/// # 戻り値
///
/// 見つかった場合は`true`。
fn has_previous_token_before_block(tokens: &[Token], index: usize, text: &str) -> bool {
    let start = tokens[..index]
        .iter()
        .rposition(|token| matches!(token.text.as_str(), ";" | "}"))
        .map_or(0, |position| position + 1);
    tokens[start..index].iter().any(|token| token.text == text)
}

/// 型修飾子として扱うトークンかを返す。
///
/// # 引数
///
/// - `value`: 判定対象のトークン文字列。
///
/// # 戻り値
///
/// 型修飾子として扱う場合は`true`。
fn is_type_qualifier(value: &str) -> bool {
    matches!(value, "const" | "volatile" | "restrict")
}

/// 標準ヘッダ由来の型名として扱うトークンかを返す。
///
/// # 引数
///
/// - `value`: 判定対象のトークン文字列。
///
/// # 戻り値
///
/// 型名として扱う場合は`true`。
///
/// TODO: typedef名をSymbol Tableで管理する。
fn is_known_type_name(value: &str) -> bool {
    matches!(value, "size_t" | "int_fast16_t")
}

/// 予約済み外部結合識別子を関数定義名として扱っているかを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象のトークン位置。
///
/// # 戻り値
///
/// 標準ライブラリの予約済み外部結合識別子として扱う場合は`true`。
///
/// TODO: 標準ライブラリ識別子一覧を環境モデルへ移す。
fn is_reserved_external_identifier(tokens: &[Token], index: usize) -> bool {
    is_reserved_external_name(&tokens[index].text)
        && tokens.get(index + 1).is_some_and(|token| token.text == "(")
        && (is_previous_declaration_keyword(tokens, index)
            || is_pointer_declaration_name(tokens, index))
        && !is_ex1_allowed_standard_function_declaration(tokens, index)
}

/// DCL37-C-EX1で許容される標準ライブラリ関数宣言かを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象の関数名トークン位置。
///
/// # 戻り値
///
/// ヘッダで定義される型を参照しない、元の宣言と適合する関数宣言の場合は`true`。
///
/// TODO: 環境モデルで標準ライブラリ関数の適合プロトタイプを管理する。
fn is_ex1_allowed_standard_function_declaration(tokens: &[Token], index: usize) -> bool {
    tokens[index].text == "free"
        && is_function_prototype_declaration(tokens, index)
        && is_void_return_type(tokens, index)
        && has_single_void_pointer_parameter(tokens, index)
}

/// 関数プロトタイプ宣言かを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象の関数名トークン位置。
///
/// # 戻り値
///
/// 関数宣言子の直後が`;`の場合は`true`。
fn is_function_prototype_declaration(tokens: &[Token], index: usize) -> bool {
    matching_paren_index(tokens, index + 1)
        .and_then(|close_paren| tokens.get(close_paren + 1))
        .is_some_and(|token| token.text == ";")
}

/// 戻り値型が`void`かを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象の関数名トークン位置。
///
/// # 戻り値
///
/// 関数名の直前が`void`の場合は`true`。
fn is_void_return_type(tokens: &[Token], index: usize) -> bool {
    index > 0 && tokens[index - 1].text == "void"
}

/// 引数が単一の`void *`かを返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `index`: 判定対象の関数名トークン位置。
///
/// # 戻り値
///
/// 引数リストが`void *`または`void * name`の場合は`true`。
fn has_single_void_pointer_parameter(tokens: &[Token], index: usize) -> bool {
    let Some(close_paren) = matching_paren_index(tokens, index + 1) else {
        return false;
    };
    let parameter_tokens = &tokens[index + 2..close_paren];

    matches!(
        parameter_tokens,
        [void_token, pointer_token]
            if void_token.text == "void" && pointer_token.text == "*"
    ) || matches!(
        parameter_tokens,
        [void_token, pointer_token, _name_token]
            if void_token.text == "void" && pointer_token.text == "*"
    )
}

/// 対応する閉じ括弧の位置を返す。
///
/// # 引数
///
/// - `tokens`: 解析対象ソースから得たトークン列。
/// - `open_paren`: 開き括弧のトークン位置。
///
/// # 戻り値
///
/// 対応する閉じ括弧が見つかった場合は、そのトークン位置。
fn matching_paren_index(tokens: &[Token], open_paren: usize) -> Option<usize> {
    if tokens.get(open_paren).is_none_or(|token| token.text != "(") {
        return None;
    }

    let mut depth = 0usize;
    for (offset, token) in tokens[open_paren..].iter().enumerate() {
        match token.text.as_str() {
            "(" => depth += 1,
            ")" => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_paren + offset);
                }
            }
            _ => {}
        }
    }

    None
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
    value == "errno"
        || value.starts_with("__")
        || value.starts_with('_')
        || is_reserved_macro_name(value)
        || is_reserved_identifier_pattern(value)
}

/// 標準ライブラリの外部結合識別子として予約済みの名前かを返す。
///
/// # 引数
///
/// - `value`: 判定対象の識別子名。
///
/// # 戻り値
///
/// 予約済み外部結合識別子として扱う場合は`true`。
fn is_reserved_external_name(value: &str) -> bool {
    matches!(
        value,
        "aligned_alloc" | "calloc" | "free" | "malloc" | "realloc"
    )
}

/// 標準ライブラリで予約済みの代表的なマクロ名かを返す。
///
/// # 引数
///
/// - `value`: 判定対象の識別子名。
///
/// # 戻り値
///
/// 予約済みマクロ名として扱う場合は`true`。
///
/// TODO: 標準ヘッダごとの予約済みマクロ一覧を環境モデルへ移す。
fn is_reserved_macro_name(value: &str) -> bool {
    matches!(value, "SIZE_MAX")
}

/// C99の将来ライブラリ方向で予約済みの識別子パターンかを返す。
///
/// # 引数
///
/// - `value`: 判定対象の識別子名。
///
/// # 戻り値
///
/// 予約済み識別子パターンとして扱う場合は`true`。
///
/// TODO: C99 7.31の予約済みパターン一覧を環境モデルへ移す。
fn is_reserved_identifier_pattern(value: &str) -> bool {
    value.starts_with("INT") && value.ends_with("_MAX")
}
