//! TypeScriptから呼び出すWASM境界API。

use std::slice;
use std::str;

use crate::analyzer::analyze;
use crate::diagnostic::diagnostics_to_json;

/// JavaScript側からWASMメモリへ入力文字列を書き込むための領域を確保する。
///
/// # 引数
///
/// - `len`: 確保するバイト数。
///
/// # 戻り値
///
/// 戻り値はWASMメモリ内の先頭ポインタ。呼び出し側はこの領域へUTF-8バイト列を書き込み、
/// `analyze_source`へ同じポインタと長さを渡す。使い終わった領域は`dealloc`で解放する。
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buffer = Vec::<u8>::with_capacity(len);
    let pointer = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    pointer
}

/// `alloc`または`analyze_source`が返したWASMメモリ領域を解放する。
///
/// # 引数
///
/// - `pointer`: 解放するWASMメモリ上の先頭ポインタ。
/// - `len`: 解放するバッファのバイト数。
///
/// `pointer`と`len`は確保時または返却時と同じ値を指定する。
///
/// TODO: 返却バッファの容量管理方式をWASM API仕様として文書化する。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(pointer: *mut u8, len: usize) {
    if !pointer.is_null() && len > 0 {
        unsafe {
            drop(Vec::from_raw_parts(pointer, 0, len));
        }
    }
}

/// UTF-8のCソースコードを解析し、JSON形式の診断結果をWASMメモリ上に返す。
///
/// # 引数
///
/// - `pointer`: 解析対象ソースコードのUTF-8バイト列を指すWASMメモリ上のポインタ。
/// - `len`: 解析対象ソースコードのバイト数。
///
/// # 戻り値
///
/// 戻り値は上位32bitに出力ポインタ、下位32bitに出力長を詰めた値。TypeScript側では
/// この値を分解してJSON文字列を読み取り、読み取り後に`dealloc`で返却バッファを解放する。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn analyze_source(pointer: *const u8, len: usize) -> u64 {
    let bytes = unsafe { slice::from_raw_parts(pointer, len) };
    let source = str::from_utf8(bytes).unwrap_or("");
    let diagnostics = analyze(source);
    let json = diagnostics_to_json(&diagnostics);
    let json_bytes = json.as_bytes();
    let mut output = Vec::<u8>::with_capacity(json_bytes.len());
    output.extend_from_slice(json_bytes);
    let output_len = output.len();
    let output_pointer = output.as_mut_ptr();
    std::mem::forget(output);
    ((output_pointer as u64) << 32) | output_len as u64
}
