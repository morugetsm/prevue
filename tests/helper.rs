#![allow(unused_imports, unused_macros)]

macro_rules! assert_render_eq {
    ($input:expr, $data:expr, $expected:expr $(,)?) => {{
        let output = prevue::render($input, $data).unwrap();
        let expected = $expected;
        assert_eq!(output, expected);
    }};
}

pub(crate) use assert_render_eq;

macro_rules! assert_render_body_eq {
    ($input:expr, $data:expr, $expected_body:expr $(,)?) => {{
        let output = prevue::render($input, $data).unwrap();
        let expected = format!("<html><head></head><body>{}</body></html>", $expected_body);
        assert_eq!(output, expected);
    }};
}

pub(crate) use assert_render_body_eq;
