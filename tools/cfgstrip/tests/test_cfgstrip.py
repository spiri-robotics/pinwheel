import json
import textwrap
from pathlib import Path

import pytest

import cfgstrip
from cfgstrip import Report, evaluate, parse_predicate, render, rewrite_source

OFF = {"enterprise", "private", "enterprise_saml"}


def strip(src: str) -> tuple[str, Report]:
    report = Report()
    out, deleted = rewrite_source(Path("x.rs"), "x.rs", textwrap.dedent(src).encode(), OFF, report)
    assert not deleted
    return out.decode(), report


# -- predicates ---------------------------------------------------------------

@pytest.mark.parametrize("text,expected", [
    ('feature = "enterprise"', False),
    ('not(feature = "enterprise")', True),
    ('all(feature = "enterprise", feature = "parquet")', False),
    ('any(not(feature = "enterprise"), feature = "sqlx")', True),
    ('all(not(feature = "private"), feature = "oauth2")', 'feature = "oauth2"'),
    ('all(feature = "parquet", not(feature = "private"), test)', 'all(feature = "parquet", test)'),
    ('any(feature = "enterprise", feature = "kafka")', 'feature = "kafka"'),
    ('not(all(feature = "enterprise", feature = "x"))', True),
    ('target_os = "linux"', 'target_os = "linux"'),
])
def test_evaluate(text, expected):
    r = evaluate(parse_predicate(text), OFF)
    assert (r if isinstance(r, bool) else render(r)) == expected


def test_trailing_comma_and_spacing():
    assert evaluate(parse_predicate('all(\n  feature = "enterprise",\n  feature = "x",\n)'), OFF) is False


# -- items and statements -----------------------------------------------------

def test_deletes_gated_item_and_its_comment_but_not_neighbours():
    out, _ = strip('''
        use a;
        // explains the gated import
        #[cfg(feature = "enterprise")]
        use b;
        use c;
    ''')
    assert out == textwrap.dedent('''
        use a;
        use c;
    ''')


def test_unwraps_not_enterprise_and_leaves_unrelated_cfgs():
    out, _ = strip('''
        #[cfg(not(feature = "enterprise"))]
        fn caps() {}
        #[cfg(test)]
        fn t() {}
        #[cfg(feature = "parquet")]
        fn p() {}
    ''')
    assert out == textwrap.dedent('''
        fn caps() {}
        #[cfg(test)]
        fn t() {}
        #[cfg(feature = "parquet")]
        fn p() {}
    ''')


def test_rewrites_partially_known_predicates():
    out, _ = strip('''
        #[cfg(all(not(feature = "private"), feature = "oauth2"))]
        fn a() {}
    ''')
    assert '#[cfg(feature = "oauth2")]\nfn a() {}' in out


def test_multiple_attributes_group():
    out, _ = strip('''
        #[allow(unused)]
        #[cfg(feature = "private")]
        pub use crate::x_ee::*;
        #[cfg(not(feature = "private"))]
        #[allow(unused)]
        pub fn stub() {}
    ''')
    assert "x_ee" not in out and "allow(unused)]\npub fn stub" in out
    assert out.count("#[allow(unused)]") == 1


def test_statements_and_tail_expression_alternatives():
    out, _ = strip('''
        fn f() -> Router {
            #[cfg(feature = "enterprise")]
            let x = secret();
            #[cfg(feature = "enterprise")]
            {
                hidden();
            }
            Router::new().nest("/a", {
                #[cfg(feature = "enterprise")]
                {
                    gated()
                }
                #[cfg(not(feature = "enterprise"))]
                Router::new()
            })
        }
    ''')
    assert "secret" not in out and "hidden" not in out and "gated" not in out
    assert "#[cfg" not in out
    assert "Router::new().nest(\"/a\", {\n        Router::new()\n    })" in out


def test_comma_lists():
    out, _ = strip('''
        struct S { #[cfg(feature = "enterprise")] a: i32, b: i32 }
        enum E { #[cfg(feature = "enterprise")] A, B }
        fn f(#[cfg(feature = "enterprise")] x: i32, y: i32) {
            let s = S { #[cfg(feature = "enterprise")] a: 1, b: 2 };
            match y {
                #[cfg(feature = "enterprise")]
                1 => secret(),
                _ => {}
            }
        }
    ''')
    assert "a: i32" not in out and "{ b: i32 }" in out
    assert "{ B }" in out
    assert "fn f(y: i32)" in out
    assert "S { b: 2 }" in out
    assert "secret" not in out and "_ => {}" in out


# -- cfg!() -------------------------------------------------------------------

def test_if_cfg_true_false_and_else_if():
    out, _ = strip('''
        fn f() {
            if cfg!(feature = "enterprise") { ee() } else { oss() }
            if !cfg!(feature = "enterprise") { caps() }
            if cfg!(feature = "enterprise") { ee2() }
            if a { x() } else if cfg!(feature = "enterprise") { ee3() } else { y() }
            if cfg!(feature = "enterprise") && other() { ee4() }
        }
    ''')
    for gone in ("ee()", "ee2()", "ee3()", "ee4()", "other()", "cfg!"):
        assert gone not in out
    assert "{ oss() }" in out and "{ caps() }" in out
    assert "if a { x() } else { y() }" in out


def test_unreachable_tail_after_folded_return_is_removed():
    out, _ = strip('''
        fn f() -> Result<()> {
            let a = 1;
            if !cfg!(feature = "enterprise") {
                return Err(not_available());
            }
            ee_only_step(a);
            Ok(())
        }
        fn g() {
            if !cfg!(feature = "enterprise") {
                log();
            }
            still_reachable();
        }
    ''')
    assert "ee_only_step" not in out and "Ok(())" not in out
    assert "return Err(not_available());" in out and "let a = 1;" in out
    assert "still_reachable();" in out


def test_cfg_macro_in_expression_is_replaced_by_literal():
    out, _ = strip('''
        fn f() -> bool { cfg!(feature = "enterprise") || debounce() }
    ''')
    assert "false || debounce()" in out


def test_cfg_inside_lazy_static():
    out, report = strip('''
        lazy_static::lazy_static! {
            #[cfg(not(feature = "enterprise"))]
            pub static ref LIMIT: i32 = 10;
            #[cfg(feature = "enterprise")]
            pub static ref LIMIT: i32 = secret_limit();
            pub static ref OTHER: i32 = 2;
        }
    ''')
    assert out == textwrap.dedent('''
        lazy_static::lazy_static! {
            pub static ref LIMIT: i32 = 10;
            pub static ref OTHER: i32 = 2;
        }
    ''')
    assert not report.flags


def test_cfg_inside_select_arms():
    out, _ = strip('''
        tokio::select! {
            _ = a() => { on_a() }
            #[cfg(feature = "enterprise")]
            _ = gated() => {
                secret();
            }
            _ = b() => on_b(),
            #[cfg(feature = "enterprise")]
            _ = gated2() => secret2(),
            _ = c() => { on_c() },
        }
    ''')
    for gone in ("gated", "secret"):
        assert gone not in out
    for kept in ("on_a()", "on_b()", "on_c()"):
        assert kept in out


def test_cfg_macro_hidden_in_macro_is_flagged():
    _, report = strip('''
        fn f() { let v = json!({ "ee": cfg!(feature = "enterprise") }); }
    ''')
    assert any("cfg! inside a macro" in f["reason"] for f in report.flags)


def test_nested_cfg_inside_kept_branch():
    out, _ = strip('''
        fn f() {
            if !cfg!(feature = "enterprise") {
                #[cfg(feature = "enterprise")]
                gone();
                kept();
            }
        }
    ''')
    assert "gone" not in out and "kept();" in out


def test_cfg_attr():
    out, _ = strip('''
        #[cfg_attr(feature = "enterprise", derive(Secret))]
        struct A;
        #[cfg_attr(not(feature = "enterprise"), allow(dead_code))]
        struct B;
    ''')
    assert "Secret" not in out and "#[allow(dead_code)]\nstruct B;" in out


def test_inner_attribute_deletes_file():
    report = Report()
    out, deleted = rewrite_source(Path("t.rs"), "t.rs",
                                  b'#![cfg(feature = "enterprise")]\nfn t() {}\n', OFF, report)
    assert deleted and report.records[0]["action"] == "delete_file"


def test_log_never_contains_removed_text():
    _, report = strip('''
        #[cfg(feature = "enterprise")]
        fn secret_function_name_body() { let SECRET_BODY_TOKEN = 1; }
    ''')
    blob = json.dumps(report.records + report.flags)
    assert "SECRET_BODY_TOKEN" not in blob


# -- whole tree: modules and Cargo.toml --------------------------------------

def test_tree(tmp_path: Path):
    (tmp_path / "Cargo.toml").write_text(textwrap.dedent('''
        [workspace]
        members = ["a"]

        [package]
        name = "root"
        version = "0.1.0"

        [features]
        enterprise = ["a/enterprise", "dep:extra"]
        private = []
        oss = ["a/parquet"]
        ee = ["enterprise", "oss"]

        [dependencies]
        a = { path = "a", features = ["enterprise", "parquet"] }
        extra = { version = "1", optional = true }
    '''))
    (tmp_path / "src").mkdir()
    (tmp_path / "src" / "main.rs").write_text("fn main() {}\n")
    a = tmp_path / "a"
    (a / "src").mkdir(parents=True)
    (a / "Cargo.toml").write_text(textwrap.dedent('''
        [package]
        name = "a"
        version = "0.1.0"

        [features]
        enterprise = []
        parquet = []
    '''))
    (a / "src" / "lib.rs").write_text(textwrap.dedent('''
        #[cfg(feature = "enterprise")]
        mod gated;
        mod open;
    '''))
    (a / "src" / "gated.rs").write_text("pub fn g() {}\n")
    (a / "src" / "open.rs").write_text('#[cfg(feature = "enterprise")]\nmod nested;\npub fn o() {}\n')
    (a / "src" / "open").mkdir()
    (a / "src" / "open" / "nested.rs").write_text("pub fn n() {}\n")
    (a / "src" / "unrelated_orphan.rs").write_text("// upstream leftover\n")

    log = tmp_path / "log.jsonl"
    assert cfgstrip.main([str(tmp_path), "--log", str(log)]) == 0

    assert not (a / "src" / "gated.rs").exists()
    assert not (a / "src" / "open" / "nested.rs").exists()
    assert (a / "src" / "unrelated_orphan.rs").exists()
    assert (a / "src" / "lib.rs").read_text().strip() == "mod open;"

    root = (tmp_path / "Cargo.toml").read_text()
    assert "enterprise" not in root.split("[dependencies]")[0].split("[features]")[1].replace('ee = ["oss"]', "")
    assert 'ee = ["oss"]' in root
    assert 'features = ["parquet"]' in root
    assert "private" not in root
    assert "enterprise" not in (a / "Cargo.toml").read_text()

    recs = [json.loads(l) for l in log.read_text().splitlines()[1:]]
    flags = [r for r in recs if r["action"] == "flag"]
    assert any(r.get("dependency") == "extra" for r in flags)
    assert any(r.get("feature") == "ee" for r in flags)

