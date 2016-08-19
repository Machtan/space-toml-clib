extern crate space_toml;
extern crate libc;
use space_toml::{Tokens, Token, TokenError};
use std::ffi::CStr;
use std::mem;
use libc::{c_char};
use std::str;
use std::panic;

pub const TOTO_ERR_INVALID_OFFSET: i32 = -3;
pub const TOTO_ERR_NULL: i32 = -2;
pub const TOTO_ERR_UTF8: i32 = -1;
pub const TOTO_NO_ERR: i32 = 0;
pub const TOTO_FINISHED: i32 = 1;
pub const TOTO_ERR: i32 = 2;

pub const TOTO_TOKEN_WHITESPACE: i32 = 1;
pub const TOTO_TOKEN_SINGLE_BRACKET_OPEN: i32 = 2;
pub const TOTO_TOKEN_DOUBLE_BRACKET_OPEN: i32 = 3;
pub const TOTO_TOKEN_SINGLE_BRACKET_CLOSE: i32 = 4;
pub const TOTO_TOKEN_DOUBLE_BRACKET_CLOSE: i32 = 5;
pub const TOTO_TOKEN_CURLY_OPEN: i32 = 6;
pub const TOTO_TOKEN_CURLY_CLOSE: i32 = 7;
pub const TOTO_TOKEN_COMMENT: i32 = 8;
pub const TOTO_TOKEN_EQUALS: i32 = 9;
pub const TOTO_TOKEN_COMMA: i32 = 10;
pub const TOTO_TOKEN_DOT: i32 = 11;
pub const TOTO_TOKEN_NEWLINE: i32 = 12;
pub const TOTO_TOKEN_KEY: i32 = 13;
pub const TOTO_TOKEN_STRING: i32 = 14;
pub const TOTO_TOKEN_MULTILINE_STRING: i32 = 15;
pub const TOTO_TOKEN_LITERAL: i32 = 16;
pub const TOTO_TOKEN_MULTILINE_LITERAL: i32 = 17;
pub const TOTO_TOKEN_DATETIME: i32 = 18;
pub const TOTO_TOKEN_INT: i32 = 19;
pub const TOTO_TOKEN_FLOAT: i32 = 20;
pub const TOTO_TOKEN_TRUE: i32 = 21;
pub const TOTO_TOKEN_FALSE: i32 = 22;

fn get_token_type(token: Token) -> i32 {
    use space_toml::Token::*;
    match token {
        Whitespace(_) => TOTO_TOKEN_WHITESPACE,
        SingleBracketOpen => TOTO_TOKEN_SINGLE_BRACKET_OPEN,
        DoubleBracketOpen => TOTO_TOKEN_DOUBLE_BRACKET_OPEN,
        SingleBracketClose => TOTO_TOKEN_SINGLE_BRACKET_CLOSE,
        DoubleBracketClose => TOTO_TOKEN_DOUBLE_BRACKET_CLOSE,
        CurlyOpen => TOTO_TOKEN_CURLY_OPEN,
        CurlyClose => TOTO_TOKEN_CURLY_CLOSE,
        Comment(_) => TOTO_TOKEN_COMMENT,
        Equals => TOTO_TOKEN_EQUALS,
        Comma => TOTO_TOKEN_COMMA,
        Dot => TOTO_TOKEN_DOT,
        Newline(_) => TOTO_TOKEN_NEWLINE,
        Key(_) => TOTO_TOKEN_KEY,
        String { text: _, literal: false, multiline: false } => TOTO_TOKEN_STRING,
        String { text: _, literal: false, multiline: true } => TOTO_TOKEN_MULTILINE_STRING,
        String { text: _, literal: true, multiline: false } => TOTO_TOKEN_LITERAL,
        String { text: _, literal: true, multiline: true } => TOTO_TOKEN_MULTILINE_LITERAL,
        DateTime(_) => TOTO_TOKEN_DATETIME,
        Int(_) => TOTO_TOKEN_INT,
        Float(_) => TOTO_TOKEN_FLOAT,
        Bool(true) => TOTO_TOKEN_TRUE,
        Bool(false) => TOTO_TOKEN_FALSE,
    }
}

fn get_token_text<'a>(token: Token<'a>) -> Option<&'a str> {
    use space_toml::Token::*;
    match token {
        Whitespace(text) => Some(text),
        Comment(text) => Some(text),
        Newline(text) => Some(text),
        Key(text) => Some(text),
        String { text, literal: _, multiline: _ } => Some(text),
        DateTime(text) => Some(text),
        Int(text) => Some(text),
        Float(text) => Some(text),
        _ => None,
    }
}

unsafe fn convert_str<'a>(text: *const c_char) -> Result<&'a str, i32> {
    if text.is_null() {
        return Err(TOTO_ERR_NULL);
    }
    let slice = CStr::from_ptr(text);
    match slice.to_str() {
        Ok(s) => Ok(s),
        Err(_) => Err(TOTO_ERR_UTF8),
    }
}

#[no_mangle]
pub unsafe extern "C" fn toto_tokenizer_new(source: *const c_char, 
        tokenizer: *mut *mut Tokens) -> i32 {
    match convert_str(source) {
        Ok(s) => {
            let tokens =  Box::new(space_toml::tokens(s));
            *tokenizer = Box::into_raw(tokens);
            TOTO_NO_ERR
        }
        Err(e) => e,
    }
}

#[no_mangle]
pub unsafe extern "C" fn toto_tokenizer_next(tokenizer: *mut Tokens, 
        token_type: *mut i32, has_text: *mut i32, text: *mut *const c_char,
        len: *mut usize, start: *mut usize, has_error: *mut i32, error: *mut *const TokenError) 
        -> i32 {
    if tokenizer.is_null() {
        return TOTO_ERR_NULL;
    }
    let tokenizer = &mut *tokenizer;
    *has_error = 0;
    if let Some(res) = tokenizer.next() {
        mem::forget(tokenizer);
        match res {
            Err(err) => {
                let e = Box::new(err);
                *error = Box::into_raw(e);
                *has_error = 1;
                TOTO_ERR
            }
            Ok((token_start, token)) => {
                *start = token_start;
                *token_type = get_token_type(token);
                if let Some(token_text) = get_token_text(token) {
                    *has_text = 1;
                    *text = token_text.as_ptr() as *const c_char;
                    *len = token_text.len();
                } else {
                    *has_text = 0;
                }
                TOTO_NO_ERR
            }
        }
    } else {
        mem::forget(tokenizer);
        TOTO_FINISHED
    }
}

#[no_mangle]
pub unsafe extern "C" fn toto_debug_get_position(text: *const c_char, 
        byte_offset: usize, col: *mut usize, row: *mut usize) -> i32 {
    match convert_str(text) {
        Ok(text) => {
            let res = panic::catch_unwind(move || {
                let (c, r) = space_toml::debug::get_position(text, byte_offset);
                *col = c;
                *row = r;
                TOTO_NO_ERR
            });
            res.unwrap_or(TOTO_ERR_INVALID_OFFSET)
        }
        Err(e) => e,
    }
}

#[no_mangle]
pub unsafe extern "C" fn toto_debug_show_unclosed(text: *const c_char, 
        start: usize) -> i32 {
    match convert_str(text) {
        Ok(text) => {
            let res = panic::catch_unwind(move || {
                space_toml::debug::show_unclosed(text, start);
                TOTO_NO_ERR
            });
            res.unwrap_or(TOTO_ERR_INVALID_OFFSET)
        }
        Err(e) => e,
    }
}

#[no_mangle]
pub unsafe extern "C" fn toto_debug_show_invalid_character(text: *const c_char, 
        pos: usize) -> i32 {
    match convert_str(text) {
        Ok(text) => {
            let res = panic::catch_unwind(move || {
                space_toml::debug::show_invalid_character(text, pos);
                TOTO_NO_ERR
            });
            res.unwrap_or(TOTO_ERR_INVALID_OFFSET)
        }
        Err(e) => e,
    }
}

#[no_mangle]
pub unsafe extern "C" fn toto_debug_show_invalid_part(text: *const c_char, 
        start: usize, pos: usize) -> i32 {
    match convert_str(text) {
        Ok(text) => {
            let res = panic::catch_unwind(move || {
                space_toml::debug::show_invalid_part(text, start, pos);
                TOTO_NO_ERR
            });
            res.unwrap_or(TOTO_ERR_INVALID_OFFSET)
        }
        Err(e) => e,
    }
}

#[no_mangle]
pub unsafe extern "C" fn toto_tokenizer_destroy(tokenizer: *mut Tokens) 
        -> i32 {
    if tokenizer.is_null() {
        return TOTO_ERR_NULL;
    }
    let _ = Box::from_raw(tokenizer);
    TOTO_NO_ERR
}

#[no_mangle]
pub unsafe extern "C" fn toto_error_explain(error: *mut TokenError, source: *const c_char) 
        -> i32 {
    if source.is_null() || error.is_null() {
        return TOTO_ERR_NULL;
    }    
    let res = str::from_utf8(CStr::from_ptr(source).to_bytes());
    let text = if let Ok(text) = res {
        text
    } else {
        return TOTO_ERR_UTF8;
    };
    let err = Box::from_raw(error);
    err.show(text);
    mem::forget(err);
    TOTO_NO_ERR
}

#[no_mangle]
pub unsafe extern "C" fn toto_error_destroy(error: *mut TokenError) -> i32 {
    if error.is_null() {
        return TOTO_ERR_NULL;
    }
    let _ = Box::from_raw(error);
    TOTO_NO_ERR
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
