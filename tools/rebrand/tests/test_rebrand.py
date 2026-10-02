import textwrap

import pytest

import rebrand
from rebrand import REPO, Report, rewrite_source


def run(rel: str, src: str) -> tuple[str, Report]:
    report = Report()
    out = rewrite_source(rel, textwrap.dedent(src).encode(), report)
    return out.decode(), report


def ts(src: str) -> str:
    return run("x.ts", src)[0]


def svelte(src: str) -> str:
    return run("x.svelte", src)[0]


# -- the name in prose ---------------------------------------------------------

@pytest.mark.parametrize("before,after", [
    ("'Enable Windmill AI here'", "'Enable Pinwheel AI here'"),
    ('"Windmill\'s database"', '"Pinwheel\'s database"'),
    ("'Windmill'", "'Pinwheel'"),
    ("`Windmill ${x} is ready`", "`Pinwheel ${x} is ready`"),
    ("'Authenticate using Windmill-signed JWTs'", "'Authenticate using Pinwheel-signed JWTs'"),
    ("'not connected to a windmill backend'", "'not connected to a pinwheel backend'"),
    ("'WINDMILL LANGUAGE CONTEXT'", "'PINWHEEL LANGUAGE CONTEXT'"),
])
def test_prose_is_renamed(before, after):
    assert ts(f"const a = {before}\n") == f"const a = {after}\n"


@pytest.mark.parametrize("src", [
    "import x from 'windmill-client'\n",
    "const a = 'windmill'\n",                      # an enum value, a mount path
    "const a = 'windmill:ctx'\n",                  # a message tag
    "const a = '[windmill] Previous logs saved'\n",  # a marker the backend writes
    "const a = 'X-Windmill-Deploy-Origin'\n",      # a header
    "const a = 'u/windmill/script'\n",
    "const a = 'cannot be called \"windmill\" here'\n",
    "if (kind === 'Windmill') {}\n",
    "switch (k) { case 'Windmill': break }\n",
    "const a = { 'Windmill': 1 }\n",
    "const a = { value: 'Windmill' }\n",
    "type T = 'Windmill' | 'other'\n",
    "s.includes('Windmill')\n",
    "localStorage.getItem('Windmill state')\n",
    "// Windmill does things\n",
    "/* Windmill */\n",
    "const r = /Windmill App/\n",
    "const WindmillIcon = 1\n",
])
def test_code_is_left_alone(src):
    assert ts(src) == src


def test_template_substitutions_are_scanned():
    assert ts("const a = `x ${ok ? 'Windmill AI' : y}`\n") == "const a = `x ${ok ? 'Pinwheel AI' : y}`\n"


# -- Svelte ---------------------------------------------------------------------

def test_svelte_markup_attributes_scripts_and_expressions():
    src = """\
    <script lang="ts">
      let t = 'Windmill AI'
      // Windmill comment
    </script>
    <svelte:head><title>Windmill</title></svelte:head>
    <div title="Go to Windmill" class="windmill-x" id="Windmill">Powered by Windmill {name}</div>
    {#if a === 'Windmill'}<b>{ok ? 'Windmill rocks' : ''}</b>{/if}
    <!-- Windmill -->
    <style>.Windmill {}</style>
    """
    out = svelte(src)
    assert "let t = 'Pinwheel AI'" in out
    assert "// Windmill comment" in out
    assert "<title>Pinwheel</title>" in out
    assert 'title="Go to Pinwheel"' in out
    assert 'class="windmill-x" id="Windmill"' in out
    assert "Powered by Pinwheel {name}" in out
    assert "a === 'Windmill'" in out
    assert "'Pinwheel rocks'" in out
    assert "<!-- Windmill -->" in out
    assert ".Windmill {}" in out


def test_multibyte_text_keeps_offsets():
    assert svelte("<p>Café — Windmill · ok</p>\n") == "<p>Café — Pinwheel · ok</p>\n"


# -- links and flags ------------------------------------------------------------

@pytest.mark.parametrize("url,target", [
    ("https://www.windmill.dev/docs/core_concepts/worker_groups#tag", REPO),
    ("https://www.windmill.dev/changelog/new-thing", f"{REPO}/releases"),
    ("https://windmill.dev", REPO),
    ("https://www.windmill.dev/blog/post", REPO),
])
def test_doc_links_point_at_the_repo(url, target):
    assert svelte(f'<a href="{url}">docs</a>\n') == f'<a href="{target}">docs</a>\n'


@pytest.mark.parametrize("src,match", [
    ("const h = 'https://hub.windmill.dev'\n", "https://hub.windmill.dev"),
    ("const a = 'Login with admin@windmill.dev please'\n", "admin@windmill.dev"),
    ("const a = 'See https://www.windmill.dev/pricing'\n", "https://www.windmill.dev/pricing"),
    ("const a = 'Get it from the Windmill Hub'\n", "Windmill Hub"),
    ("const a = 'Windmill Enterprise Edition only'\n", "Windmill Enterprise"),
    ("const a = `https://www.windmill.dev/docs/${p}`\n", "https://www.windmill.dev/docs/"),
    ("if (host === 'app.windmill.dev') {}\n", "app.windmill.dev"),
])
def test_unfixable_mentions_are_flagged_not_rewritten(src, match):
    out, report = run("x.ts", textwrap.dedent(src))
    assert out == src
    assert [f["match"] for f in report.flags] == [match]


def test_tests_and_generated_files_are_out_of_scope():
    assert rebrand.relevant("src/lib/a.svelte")
    assert rebrand.relevant("static/openapi2.html")
    assert not rebrand.relevant("src/lib/a.test.ts")
    assert not rebrand.relevant("src/lib/x.gen.ts")
    assert not rebrand.relevant("src/lib/gen/types.ts")
    assert not rebrand.relevant("e2e/login.spec.ts")
    assert not rebrand.relevant("node_modules/x/index.js")


# -- check mode -----------------------------------------------------------------

def test_check_fails_on_pending_rewrites_and_new_flags(tmp_path, capsys):
    root = tmp_path / "frontend"
    (root / "src").mkdir(parents=True)
    page = root / "src" / "a.svelte"
    known = tmp_path / "known.txt"

    page.write_text("<p>Windmill</p>\n")
    assert rebrand.main([str(root), "--check", "--known", str(known)]) == 1
    assert page.read_text() == "<p>Windmill</p>\n"  # --check changes nothing
    assert rebrand.main([str(root), "--known", str(known)]) == 0
    assert page.read_text() == "<p>Pinwheel</p>\n"
    assert rebrand.main([str(root), "--check", "--known", str(known)]) == 0

    page.write_text("<p>From the Windmill Hub</p>\n")
    assert rebrand.main([str(root), "--check", "--known", str(known)]) == 1
    assert rebrand.main([str(root), "--check", "--known", str(known), "--update-known"]) == 0
    assert "Windmill Hub" in known.read_text()
    assert rebrand.main([str(root), "--check", "--known", str(known)]) == 0

    # a second, unacknowledged occurrence of the same flag still fails
    page.write_text("<p>From the Windmill Hub</p>\n<p>Windmill Hub</p>\n")
    assert rebrand.main([str(root), "--check", "--known", str(known)]) == 1
