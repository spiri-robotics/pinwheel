"""cfgstrip: remove Cargo-feature-gated Rust code from a source tree.

Every `#[cfg(...)]`, `#[cfg_attr(...)]` and `cfg!(...)` predicate is partially
evaluated with a chosen set of features forced OFF; everything else stays
unknown. Code whose predicate is now always false is deleted, attributes that
are now always true are dropped, and the rest are rewritten in simplified form.
Cargo.toml feature tables are updated to match, and module files that only an
always-false `mod` declaration pulled in are deleted.

The tool works blind by design: its log records *where* it cut (file, lines,
enclosing item, the predicate) but never the text it removed, so the person
running it never has to read the stripped code.

Usage:
    cfgstrip ROOT [--off FEATURE ...] [--log FILE] [--dry-run]

ROOT is a Cargo workspace (or a directory containing several). With no --off,
the features `enterprise`, `private` and `enterprise_saml` are switched off.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import posixpath
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path, PurePosixPath

import tomlkit
import tree_sitter_rust
from tree_sitter import Language, Node, Parser

__version__ = "0.2.0"
DEFAULT_OFF = ("enterprise", "private", "enterprise_saml")
RUST = Language(tree_sitter_rust.language())

# ---------------------------------------------------------------------------
# cfg predicates
# ---------------------------------------------------------------------------
# A predicate is one of:
#   ("feat", name)        feature = "name"
#   ("atom", text)        any other leaf (`test`, `unix`, `target_os = "linux"`)
#   ("all", [preds])  ("any", [preds])  ("not", pred)
# Partial evaluation returns True, False, or a (simplified) predicate.

_TOKEN = re.compile(r'\s*(?:(r#)?([A-Za-z_][A-Za-z0-9_]*)|("(?:\\.|[^"\\])*")|([=(),]))')


class PredicateError(ValueError):
    pass


def _tokenize(text: str) -> list[str]:
    out, pos = [], 0
    text = text.strip()
    while pos < len(text):
        m = _TOKEN.match(text, pos)
        if not m or m.end() == pos:
            if text[pos:].strip() == "":
                break
            raise PredicateError(f"unexpected input in cfg predicate at {text[pos:pos + 20]!r}")
        out.append(m.group(2) or m.group(3) or m.group(4))
        pos = m.end()
    return out


def parse_predicate(text: str):
    toks = _tokenize(text)
    pos = 0

    def peek():
        return toks[pos] if pos < len(toks) else None

    def take(expected=None):
        nonlocal pos
        tok = peek()
        if tok is None or (expected is not None and tok != expected):
            raise PredicateError(f"expected {expected!r}, got {tok!r} in cfg({text})")
        pos += 1
        return tok

    def pred():
        name = take()
        if not re.match(r"[A-Za-z_]", name):
            raise PredicateError(f"unexpected {name!r} in cfg({text})")
        if peek() == "(" and name in ("all", "any", "not"):
            take("(")
            items = []
            while peek() != ")":
                items.append(pred())
                if peek() == ",":
                    take(",")
            take(")")
            if name == "not":
                if len(items) != 1:
                    raise PredicateError(f"not() takes one predicate in cfg({text})")
                return ("not", items[0])
            return (name, items)
        if peek() == "=":
            take("=")
            value = take()
            if name == "feature":
                return ("feat", value[1:-1])
            return ("atom", f"{name} = {value}")
        return ("atom", name)

    result = pred()
    if peek() == ",":  # tolerate a trailing comma
        take(",")
    if peek() is not None:
        raise PredicateError(f"trailing input in cfg({text})")
    return result


def mentions(p, off: set[str]) -> bool:
    kind = p[0]
    if kind == "feat":
        return p[1] in off
    if kind == "atom":
        return False
    if kind == "not":
        return mentions(p[1], off)
    return any(mentions(q, off) for q in p[1])


def evaluate(p, off: set[str]):
    kind = p[0]
    if kind == "feat":
        return False if p[1] in off else p
    if kind == "atom":
        return p
    if kind == "not":
        inner = evaluate(p[1], off)
        if inner is True:
            return False
        if inner is False:
            return True
        return ("not", inner)
    parts = []
    for q in p[1]:
        r = evaluate(q, off)
        if kind == "all":
            if r is False:
                return False
            if r is not True:
                parts.append(r)
        else:
            if r is True:
                return True
            if r is not False:
                parts.append(r)
    if not parts:
        return kind == "all"
    if len(parts) == 1:
        return parts[0]
    return (kind, parts)


def render(p) -> str:
    kind = p[0]
    if kind == "feat":
        return f'feature = "{p[1]}"'
    if kind == "atom":
        return p[1]
    if kind == "not":
        return f"not({render(p[1])})"
    return f"{kind}({', '.join(render(q) for q in p[1])})"


# ---------------------------------------------------------------------------
# Edits and logging
# ---------------------------------------------------------------------------


@dataclass
class Edit:
    start: int
    end: int
    replacement: bytes
    record: dict


@dataclass
class Report:
    records: list[dict] = field(default_factory=list)
    flags: list[dict] = field(default_factory=list)

    def add(self, rec: dict) -> None:
        (self.flags if rec["action"] == "flag" else self.records).append(rec)


def line_of(src: bytes, offset: int) -> int:
    return src.count(b"\n", 0, offset) + 1


def enclosing(node: Node) -> str:
    """Names of the items around `node`, outermost first (e.g. `impl Foo::bar`)."""
    names = []
    n = node.parent
    while n is not None:
        if n.type in ("function_item", "mod_item", "struct_item", "enum_item", "trait_item",
                      "function_signature_item", "macro_definition"):
            name = n.child_by_field_name("name")
            if name is not None:
                names.append(name.text.decode())
        elif n.type == "impl_item":
            t = n.child_by_field_name("type")
            if t is not None:
                names.append(f"impl {t.text.decode()}")
        n = n.parent
    return "::".join(reversed(names)) or "<file>"


def describe(node: Node) -> str:
    """Kind and name of a deleted node, without its body."""
    name = node.child_by_field_name("name")
    if name is not None:
        return f"{node.type} {name.text.decode()}"
    return node.type


# ---------------------------------------------------------------------------
# Rust source rewriting
# ---------------------------------------------------------------------------

# Node types whose attributes are leading children rather than preceding siblings.
PREFIX_ATTR_PARENTS = {
    "field_initializer", "shorthand_field_initializer", "base_field_initializer",
    "match_arm", "field_pattern",
}
SKIP_SIBLINGS = {"attribute_item", "line_comment", "block_comment"}


def attr_kind(attr_item: Node):
    """Return ("cfg"|"cfg_attr", argument_text) for a cfg attribute item, else None."""
    attr = next((c for c in attr_item.named_children if c.type == "attribute"), None)
    if attr is None:
        return None
    path = attr.named_children[0] if attr.named_children else None
    args = attr.child_by_field_name("arguments")
    if path is None or args is None or path.type != "identifier":
        return None
    name = path.text.decode()
    if name not in ("cfg", "cfg_attr"):
        return None
    text = args.text.decode()
    return name, text[1:-1]  # strip the surrounding parens


def split_cfg_attr(text: str) -> tuple[str, str]:
    """Split `pred, attr1, attr2` at the first top-level comma."""
    depth = 0
    in_str = False
    i = 0
    while i < len(text):
        ch = text[i]
        if in_str:
            if ch == "\\":
                i += 1
            elif ch == '"':
                in_str = False
        elif ch == '"':
            in_str = True
        elif ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif ch == "," and depth == 0:
            return text[:i], text[i + 1:].strip()
        i += 1
    raise PredicateError(f"cfg_attr without attributes: {text!r}")


def find_target(attr_item: Node) -> tuple[Node | None, list[Node]]:
    """The node an attribute applies to, plus the full group of its attributes."""
    parent = attr_item.parent
    if parent is not None and parent.type in PREFIX_ATTR_PARENTS:
        group = [c for c in parent.children if c.type == "attribute_item"]
        return parent, group
    sib = attr_item.next_sibling
    while sib is not None and (sib.type in SKIP_SIBLINGS or not sib.is_named):
        if not sib.is_named and sib.type not in SKIP_SIBLINGS:
            return None, [attr_item]  # attribute followed by punctuation: give up
        sib = sib.next_sibling
    if sib is None:
        return None, [attr_item]
    group = []
    prev = sib.prev_sibling
    while prev is not None and prev.type in SKIP_SIBLINGS:
        if prev.type == "attribute_item":
            group.append(prev)
        prev = prev.prev_sibling
    group.reverse()
    return sib, group


def widen_delete(src: bytes, start: int, end: int, node: Node | None) -> tuple[int, int]:
    """Grow a deletion to whole lines, eat a trailing comma and adjacent comments."""
    # trailing comma in comma-separated lists
    if node is not None:
        nxt = node.next_sibling
        if nxt is not None and not nxt.is_named and nxt.type == ",":
            end = nxt.end_byte
    line_start = src.rfind(b"\n", 0, start) + 1
    if src[line_start:start].strip() == b"":
        start = line_start
        # absorb directly preceding `//` comment lines (no blank line between)
        while start > 0:
            prev_start = src.rfind(b"\n", 0, start - 1) + 1
            prev = src[prev_start:start - 1].strip()
            if prev.startswith(b"//") and not prev.startswith(b"//!"):
                start = prev_start
            else:
                break
    line_end = src.find(b"\n", end)
    if line_end == -1:
        line_end = len(src)
    if src[end:line_end].strip() == b"":
        end = min(line_end + 1, len(src))
    else:  # inline deletion: also eat the spaces that separated it from what follows
        while end < len(src) and src[end:end + 1] in (b" ", b"\t"):
            end += 1
    return start, end


def widen_attr(src: bytes, start: int, end: int) -> tuple[int, int]:
    line_start = src.rfind(b"\n", 0, start) + 1
    line_end = src.find(b"\n", end)
    if line_end == -1:
        line_end = len(src)
    if src[line_start:start].strip() == b"" and src[end:line_end].strip() == b"":
        return line_start, min(line_end + 1, len(src))
    # inline attribute: also eat one following space
    if src[end:end + 1] == b" ":
        end += 1
    return start, end


class FileRewriter:
    def __init__(self, path: Path, rel: str, src: bytes, off: set[str]):
        self.path = path
        self.rel = rel
        self.src = src
        self.off = off
        self.delete_file = False

    # -- one pass -----------------------------------------------------------
    def collect(self, tree, report: Report) -> list[Edit]:
        edits: list[Edit] = []
        claimed: list[tuple[int, int]] = []
        seen_targets: set[tuple[int, int]] = set()
        src = self.src

        def free(a: int, b: int) -> bool:
            return all(b <= s or a >= e for s, e in claimed)

        def base(node: Node, action: str, **extra) -> dict:
            return {"action": action, "file": self.rel,
                    "start_line": node.start_point[0] + 1,
                    "end_line": node.end_point[0] + 1,
                    "enclosing": enclosing(node), **extra}

        def visit(node: Node):
            # skip anything already swallowed by an earlier edit in this pass
            if any(s <= node.start_byte and node.end_byte <= e for s, e in claimed):
                return
            if node.type == "inner_attribute_item" and node.parent is not None \
                    and node.parent.type == "source_file":
                self.handle_inner(node, edits, claimed, report, base)
            elif node.type == "attribute_item":
                self.handle_attr(node, edits, claimed, seen_targets, report, base, free)
            elif node.type == "if_expression":
                self.handle_if(node, edits, claimed, report, base, free)
            elif node.type == "macro_invocation":
                self.handle_macro(node, edits, claimed, report, base, free)
            if node.has_error and node.type == "ERROR":
                report.add(base(node, "flag", reason="parse error region; cfgs inside not processed"))
                return
            for child in node.children:
                visit(child)

        visit(tree.root_node)
        return edits

    # -- handlers -----------------------------------------------------------
    def handle_inner(self, node, edits, claimed, report, base):
        kind = attr_kind_inner(node)
        if kind is None:
            return
        name, text = kind
        if name != "cfg":
            return
        try:
            pred = parse_predicate(text)
        except PredicateError as e:
            report.add(base(node, "flag", reason=str(e)))
            return
        if not mentions(pred, self.off):
            return
        result = evaluate(pred, self.off)
        if result is False:
            self.delete_file = True
            report.add({"action": "delete_file", "file": self.rel,
                        "lines": self.src.count(b"\n") + 1,
                        "predicate": f"cfg({text})", "reason": "file-level cfg is always false"})
            claimed.append((0, len(self.src)))
            return
        if result is True:
            s, e = widen_attr(self.src, node.start_byte, node.end_byte)
            repl, action, shown = b"", "unwrap", True
        else:
            s, e = node.start_byte, node.end_byte
            repl, action, shown = f"#![cfg({render(result)})]".encode(), "rewrite", render(result)
        edits.append(Edit(s, e, repl, base(node, action, predicate=f"cfg({text})", result=shown)))
        claimed.append((s, e))

    def handle_attr(self, node, edits, claimed, seen, report, base, free):
        kind = attr_kind(node)
        if kind is None:
            return
        target, group = find_target(node)
        if target is None:
            if mentions_any_text(kind[1], self.off):
                report.add(base(node, "flag", reason="cfg attribute with no recognisable target",
                                predicate=f"{kind[0]}({kind[1]})"))
            return
        key = (target.start_byte, target.end_byte)
        if key in seen:
            return
        seen.add(key)

        cfgs = []  # (attr_node, name, text, pred)
        for a in group:
            k = attr_kind(a)
            if k is None:
                continue
            name, text = k
            try:
                ptext = text if name == "cfg" else split_cfg_attr(text)[0]
                pred = parse_predicate(ptext)
            except PredicateError as e:
                if mentions_any_text(text, self.off):
                    report.add(base(a, "flag", reason=str(e)))
                continue
            cfgs.append((a, name, text, pred))
        relevant = [c for c in cfgs if mentions(c[3], self.off)]
        if not relevant:
            return

        # Any plain cfg that is now always false removes the whole target.
        for a, name, text, pred in cfgs:
            if name == "cfg" and evaluate(pred, self.off) is False:
                first = group[0] if target.type not in PREFIX_ATTR_PARENTS else target
                start = min(first.start_byte, target.start_byte)
                s, e = widen_delete(self.src, start, target.end_byte, target)
                if not free(s, e):
                    return
                edits.append(Edit(s, e, b"", base(
                    target, "delete", item=describe(target),
                    lines=line_of(self.src, e - 1) - line_of(self.src, s) + 1,
                    predicate=f"cfg({text})")))
                claimed.append((s, e))
                return

        for a, name, text, pred in relevant:
            result = evaluate(pred, self.off)
            if name == "cfg":
                if result is True:
                    s, e = widen_attr(self.src, a.start_byte, a.end_byte)
                    repl, action = b"", "unwrap"
                else:
                    s, e = a.start_byte, a.end_byte
                    repl, action = f"#[cfg({render(result)})]".encode(), "rewrite"
            else:  # cfg_attr
                _, attrs = split_cfg_attr(text)
                if result is False:
                    s, e = widen_attr(self.src, a.start_byte, a.end_byte)
                    repl, action = b"", "drop_cfg_attr"
                elif result is True:
                    s, e = a.start_byte, a.end_byte
                    repl, action = f"#[{attrs}]".encode(), "unwrap"
                else:
                    s, e = a.start_byte, a.end_byte
                    repl, action = f"#[cfg_attr({render(result)}, {attrs})]".encode(), "rewrite"
            if not free(s, e):
                continue
            edits.append(Edit(s, e, repl, base(
                a, action, predicate=f"{name}({text if name == 'cfg' else split_cfg_attr(text)[0]})",
                result=True if result is True else (False if result is False else render(result)))))
            claimed.append((s, e))

    def handle_macro_tokens(self, node, edits, claimed, report, base, free) -> bool:
        """Rewrite cfg attributes found at token level inside a macro invocation."""
        found = False
        mac = node.child_by_field_name("macro")
        mac_name = mac.text.decode() if mac is not None else "?"
        stack = [c for c in node.named_children if c.type == "token_tree"]
        while stack:
            tt = stack.pop()
            kids = tt.children
            stack.extend(c for c in kids if c.type == "token_tree")
            i = 0
            while i < len(kids):
                first = _token_attr(kids, i)
                if first is None and not (kids[i].type == "#" and i + 1 < len(kids)
                                          and kids[i + 1].type == "token_tree"
                                          and kids[i + 1].text.startswith(b"[")):
                    i += 1
                    continue
                # gather the consecutive attribute group starting at i
                group, j = [], i
                while j + 1 < len(kids) and kids[j].type == "#" and kids[j + 1].type == "token_tree" \
                        and kids[j + 1].text.startswith(b"["):
                    group.append((j, _token_attr(kids, j)))
                    j += 2
                cfgs = []
                for idx, info in group:
                    if info is None:
                        continue
                    name, text, _ = info
                    try:
                        pred = parse_predicate(text if name == "cfg" else split_cfg_attr(text)[0])
                    except PredicateError:
                        continue
                    cfgs.append((idx, name, text, pred))
                relevant = [c for c in cfgs if mentions(c[3], self.off)]
                if not relevant:
                    i = j
                    continue
                found = True
                rec_extra = {"macro": mac_name}
                false_cfg = next((c for c in cfgs if c[1] == "cfg" and evaluate(c[3], self.off) is False), None)
                if false_cfg is not None:
                    end_idx = _token_item_end(kids, j)
                    s, e = widen_delete(self.src, kids[i].start_byte, kids[end_idx].end_byte, None)
                    if free(s, e):
                        edits.append(Edit(s, e, b"", {
                            "action": "delete", "file": self.rel,
                            "start_line": kids[i].start_point[0] + 1,
                            "end_line": kids[end_idx].end_point[0] + 1,
                            "enclosing": enclosing(node), "item": f"tokens in {mac_name}!",
                            "lines": line_of(self.src, e - 1) - line_of(self.src, s) + 1,
                            "predicate": f"cfg({false_cfg[2]})", **rec_extra}))
                        claimed.append((s, e))
                    i = end_idx + 1
                    continue
                for idx, name, text, pred in relevant:
                    result = evaluate(pred, self.off)
                    a_start, a_end = kids[idx].start_byte, kids[idx + 1].end_byte
                    if name == "cfg":
                        if result is True:
                            s, e = widen_attr(self.src, a_start, a_end)
                            repl, action = b"", "unwrap"
                        else:
                            s, e = a_start, a_end
                            repl, action = f"#[cfg({render(result)})]".encode(), "rewrite"
                    else:
                        _, attrs = split_cfg_attr(text)
                        if result is False:
                            s, e = widen_attr(self.src, a_start, a_end)
                            repl, action = b"", "drop_cfg_attr"
                        elif result is True:
                            s, e = a_start, a_end
                            repl, action = f"#[{attrs}]".encode(), "unwrap"
                        else:
                            s, e = a_start, a_end
                            repl, action = f"#[cfg_attr({render(result)}, {attrs})]".encode(), "rewrite"
                    if not free(s, e):
                        continue
                    edits.append(Edit(s, e, repl, {
                        "action": action, "file": self.rel,
                        "start_line": kids[idx].start_point[0] + 1,
                        "end_line": kids[idx + 1].end_point[0] + 1,
                        "enclosing": enclosing(node), "predicate": f"{name}({text})",
                        "result": result if isinstance(result, bool) else render(result), **rec_extra}))
                    claimed.append((s, e))
                i = j
        return found

    def cfg_macro_value(self, node: Node):
        """For a `cfg!(...)` macro, return (pred, result); otherwise None."""
        if node.type != "macro_invocation":
            return None
        mac = node.child_by_field_name("macro")
        if mac is None or mac.type != "identifier" or mac.text != b"cfg":
            return None
        tt = next((c for c in node.named_children if c.type == "token_tree"), None)
        if tt is None:
            return None
        try:
            pred = parse_predicate(tt.text.decode()[1:-1])
        except PredicateError:
            return None
        return pred, evaluate(pred, self.off)

    def fold(self, node: Node):
        """Constant-fold a boolean condition. Returns (value|None, touches_off)."""
        t = node.type
        if t == "parenthesized_expression":
            inner = next((c for c in node.named_children if c.type not in SKIP_SIBLINGS), None)
            return self.fold(inner) if inner is not None else (None, False)
        if t == "macro_invocation":
            mv = self.cfg_macro_value(node)
            if mv is None:
                return None, False
            pred, result = mv
            touches = mentions(pred, self.off)
            return (result if isinstance(result, bool) else None), touches
        if t == "boolean_literal":
            return node.text == b"true", False
        if t == "unary_expression" and node.children and node.children[0].type == "!":
            v, touches = self.fold(node.named_children[-1])
            return (None if v is None else not v), touches
        if t == "binary_expression":
            op = node.child_by_field_name("operator")
            left = node.child_by_field_name("left")
            right = node.child_by_field_name("right")
            if op is None or op.type not in ("&&", "||"):
                return None, False
            lv, lt = self.fold(left)
            rv, rt = self.fold(right)
            touches = lt or rt
            if op.type == "&&":
                if lv is False:
                    return False, touches  # right side was never evaluated
                if lv is True:
                    return rv, touches
            else:
                if lv is True:
                    return True, touches
                if lv is False:
                    return rv, touches
            return None, touches
        return None, False

    def handle_if(self, node, edits, claimed, report, base, free):
        cond = node.child_by_field_name("condition")
        if cond is None:
            return
        value, touches = self.fold(cond)
        if not touches or value is None:
            return
        cons = node.child_by_field_name("consequence")
        alt = node.child_by_field_name("alternative")
        branch = None
        if alt is not None:
            branch = next((c for c in alt.named_children if c.type in ("block", "if_expression")), None)
        # Cut away the condition and the dead branch but leave the kept branch
        # in place, so edits inside it still happen in this pass (and the log's
        # line numbers keep referring to the original file).
        if value:
            kept, cuts = "then-branch", [(node.start_byte, cons.start_byte, b""),
                                         (cons.end_byte, node.end_byte, b"")]
        elif branch is not None:
            kept, cuts = "else-branch", [(node.start_byte, branch.start_byte, b"")]
        else:
            kept, cuts = "nothing", [(node.start_byte, node.end_byte, b"{}")]
        cuts = [(s, e, r) for s, e, r in cuts if e > s]
        if not all(free(s, e) for s, e, _ in cuts):
            return
        rec = base(node, "if_fold", kept=kept, condition=cond.text.decode()[:200])
        for i, (s, e, r) in enumerate(cuts):
            edits.append(Edit(s, e, r, rec if i == 0 else {**rec, "action": "if_fold_tail"}))
            claimed.append((s, e))
        kept_block = cons if value else (branch if branch is not None and branch.type == "block" else None)
        if kept_block is not None and diverges(kept_block):
            self.cut_unreachable_after(node, edits, claimed, base, free)

    def cut_unreachable_after(self, node, edits, claimed, base, free):
        """Delete statements following a folded `if` whose kept branch always returns."""
        stmt = node.parent if node.parent is not None and node.parent.type == "expression_statement" else node
        block = stmt.parent
        if block is None or block.type != "block":
            return
        rest = []
        sib = stmt.next_sibling
        while sib is not None:
            if sib.type != "}":
                rest.append(sib)
            sib = sib.next_sibling
        named = [n for n in rest if n.is_named and n.type not in ("line_comment", "block_comment")]
        if not named:
            return
        s, e = widen_delete(self.src, rest[0].start_byte, named[-1].end_byte, None)
        s = max(s, stmt.end_byte)
        if not free(s, e):
            return
        edits.append(Edit(s, e, b"\n" if self.src[s - 1:s] != b"\n" else b"", base(
            named[0], "delete", item="statements unreachable after folded cfg! check",
            lines=line_of(self.src, e - 1) - line_of(self.src, s) + 1,
            predicate="(follows if_fold)")))
        claimed.append((s, e))

    def handle_macro(self, node, edits, claimed, report, base, free):
        mv = self.cfg_macro_value(node)
        if mv is not None:
            pred, result = mv
            if not mentions(pred, self.off):
                return
            # inside an if condition that could not be folded, or any other position
            if result is True or result is False:
                repl = b"true" if result else b"false"
            else:
                repl = f"cfg!({render(result)})".encode()
            if not free(node.start_byte, node.end_byte):
                return
            rec = base(node, "cfg_macro", predicate=f"cfg!({render(pred)})",
                       result=result if isinstance(result, bool) else render(result))
            guard = node.parent
            while guard is not None and guard.type not in ("match_arm", "block", "source_file"):
                guard = guard.parent
            if result is False and guard is not None and guard.type == "match_arm":
                report.add(base(node, "flag", reason="cfg! in a match guard is now false; arm may be dead"))
            edits.append(Edit(node.start_byte, node.end_byte, repl, rec))
            claimed.append((node.start_byte, node.end_byte))
            return
        # cfg attributes hidden in macro token trees (lazy_static!, select!, ...)
        if node.child_by_field_name("macro") is not None and \
                node.child_by_field_name("macro").text in (b"macro_rules",):
            return
        handled = self.handle_macro_tokens(node, edits, claimed, report, base, free)
        text = node.text.decode(errors="replace")
        for m in re.finditer(r"\bcfg!\s*\(", text):
            i, depth = m.end(), 1
            while i < len(text) and depth:
                depth += {"(": 1, ")": -1}.get(text[i], 0)
                i += 1
            try:
                pred = parse_predicate(text[m.end():i - 1])
            except PredicateError:
                continue
            if mentions(pred, self.off):
                line = node.start_point[0] + 1 + text.count("\n", 0, m.start())
                report.add({"action": "flag", "file": self.rel, "start_line": line, "end_line": line,
                            "enclosing": enclosing(node), "macro": mac_text(node),
                            "predicate": f"cfg!({render(pred)})",
                            "reason": "cfg! inside a macro invocation; not rewritten"})
        if handled:
            return
        for m in re.finditer(r"#\s*!?\s*\[\s*(cfg|cfg_attr)\s*\(", text):
            # locate the matching close paren
            i, depth = m.end(), 1
            while i < len(text) and depth:
                depth += {"(": 1, ")": -1}.get(text[i], 0)
                i += 1
            inner = text[m.end():i - 1]
            ptext = inner if m.group(1) == "cfg" else inner.split(",", 1)[0]
            try:
                pred = parse_predicate(ptext)
            except PredicateError:
                continue
            if mentions(pred, self.off):
                line = node.start_point[0] + 1 + text.count("\n", 0, m.start())
                mac = node.child_by_field_name("macro")
                report.add({"action": "flag", "file": self.rel, "start_line": line, "end_line": line,
                            "enclosing": enclosing(node),
                            "macro": mac.text.decode() if mac is not None else "?",
                            "predicate": f"{m.group(1)}({ptext.strip()})",
                            "result": (lambda r: r if isinstance(r, bool) else render(r))(evaluate(pred, self.off)),
                            "reason": "cfg inside a macro invocation; not rewritten, needs manual handling"})


def _token_attr(tt_children: list[Node], i: int):
    """If tt_children[i] starts `#[cfg(...)]`/`#[cfg_attr(...)]`, return (name, args, end_index)."""
    if tt_children[i].type != "#" or i + 1 >= len(tt_children):
        return None
    br = tt_children[i + 1]
    if br.type != "token_tree" or not br.text.startswith(b"["):
        return None
    inner = [c for c in br.children if c.type not in ("[", "]")]
    if len(inner) < 2 or inner[0].type != "identifier" or inner[0].text not in (b"cfg", b"cfg_attr"):
        return None
    if inner[1].type != "token_tree" or not inner[1].text.startswith(b"("):
        return None
    return inner[0].text.decode(), inner[1].text.decode()[1:-1], i + 1


def _token_item_end(tt_children: list[Node], j: int) -> int:
    """Index of the last child belonging to the item that starts at j.

    The item ends at the first `;` or `,` at this depth (inclusive). A
    `=> { ... }` arm (tokio::select!) may omit its comma, so it also ends after
    the block that follows `=>`.
    """
    seen_arrow = False
    k = j
    while k < len(tt_children):
        c = tt_children[k]
        if c.type in (";", ","):
            return k
        if c.type == "=>":
            seen_arrow = True
        elif seen_arrow and c.type == "token_tree" and c.text.startswith(b"{"):
            if k + 1 < len(tt_children) and tt_children[k + 1].type == ",":
                return k + 1
            return k
        if c.type in ("}", ")", "]") and k == len(tt_children) - 1:
            return k - 1  # closing delimiter of the enclosing tree
        k += 1
    return len(tt_children) - 1


def diverges(block: Node) -> bool:
    """True if a block's last statement is a `return` (so code after it is unreachable)."""
    named = [c for c in block.named_children if c.type not in ("line_comment", "block_comment")]
    if not named:
        return False
    last = named[-1]
    if last.type == "expression_statement" and last.named_children:
        last = last.named_children[0]
    return last.type == "return_expression"


def mac_text(node: Node) -> str:
    mac = node.child_by_field_name("macro")
    return mac.text.decode() if mac is not None else "?"


def attr_kind_inner(node: Node):
    attr = next((c for c in node.named_children if c.type == "attribute"), None)
    if attr is None or not attr.named_children:
        return None
    path = attr.named_children[0]
    args = attr.child_by_field_name("arguments")
    if args is None or path.type != "identifier":
        return None
    return path.text.decode(), args.text.decode()[1:-1]


def mentions_any_text(text: str, off: set[str]) -> bool:
    return any(re.search(rf'"{re.escape(f)}"', text) for f in off)


def apply_edits(src: bytes, edits: list[Edit]) -> bytes:
    edits = sorted(edits, key=lambda e: e.start)
    for a, b in zip(edits, edits[1:]):
        if b.start < a.end:
            raise RuntimeError(f"overlapping edits at bytes {a.start}-{a.end} / {b.start}-{b.end}")
    out = bytearray(src)
    for e in reversed(edits):
        out[e.start:e.end] = e.replacement
    return bytes(out)


def rewrite_source(path: Path, rel: str, src: bytes, off: set[str], report: Report,
                   max_passes: int = 12) -> tuple[bytes, bool]:
    parser = Parser(RUST)
    fr = FileRewriter(path, rel, src, off)
    seen_flags = set()
    for n in range(max_passes):
        tree = parser.parse(fr.src)
        local = Report()
        edits = fr.collect(tree, local)
        # Later passes see shifted line numbers; only keep flags not already raised.
        for f in local.flags:
            key = (f.get("reason"), f.get("predicate"), f.get("enclosing"), f.get("macro"))
            if key not in seen_flags:
                seen_flags.add(key)
                report.add({**f, "pass": n + 1})
        for r in local.records:
            report.add({**r, "pass": n + 1})
        if fr.delete_file:
            return b"", True
        if not edits:
            break
        for e in edits:
            report.add({**e.record, "pass": n + 1})
        fr.src = apply_edits(fr.src, edits)
    else:
        report.add({"action": "flag", "file": rel, "reason": "did not converge"})
    return fr.src, False




# ---------------------------------------------------------------------------
# Whole-tree layer
# ---------------------------------------------------------------------------
# Everything below works on a set of relative POSIX paths plus a `read(path)`
# function, so the same code strips a directory on disk or a git tree.

_DECL_CACHE: dict[bytes, list[tuple[tuple[str, ...], str, str | None]]] = {}


def mod_decls(src: bytes) -> list[tuple[tuple[str, ...], str, str | None]]:
    """Out-of-line `mod x;` declarations in a file: (inline-mod prefix, name, #[path])."""
    key = hashlib.sha1(src).digest()
    hit = _DECL_CACHE.get(key)
    if hit is not None:
        return hit
    out: list[tuple[tuple[str, ...], str, str | None]] = []
    tree = Parser(RUST).parse(src)

    def walk(node: Node, prefix: tuple[str, ...]):
        for child in node.named_children:
            if child.type == "mod_item":
                name = child.child_by_field_name("name").text.decode()
                body = child.child_by_field_name("body")
                path_attr = None
                prev = child.prev_sibling
                while prev is not None and prev.type in SKIP_SIBLINGS:
                    if prev.type == "attribute_item":
                        m = re.match(rb'#\[\s*path\s*=\s*"([^"]+)"\s*\]', prev.text)
                        if m:
                            path_attr = m.group(1).decode()
                    prev = prev.prev_sibling
                if body is not None:
                    walk(body, prefix + (name,))
                else:
                    out.append((prefix, name, path_attr))
            elif child.type == "declaration_list":
                walk(child, prefix)

    walk(tree.root_node, ())
    _DECL_CACHE[key] = out
    return out


def _mod_rs_like(path: str, roots: set[str]) -> bool:
    return posixpath.basename(path) in ("mod.rs", "lib.rs", "main.rs") or path in roots


def module_files(roots: set[str], decls, exists) -> set[str]:
    """Files reachable from `roots` through `mod x;` declarations (cfgs ignored).

    `decls(path)` returns that file's `mod_decls`, or None if it can't be read.
    """
    seen: set[str] = set()
    stack = [r for r in roots if exists(r)]
    while stack:
        f = stack.pop()
        if f in seen:
            continue
        seen.add(f)
        found = decls(f)
        if found is None:
            continue
        parent = posixpath.dirname(f)
        base_dir = parent if _mod_rs_like(f, roots) else posixpath.join(
            parent, posixpath.splitext(posixpath.basename(f))[0])
        for prefix, name, path_attr in found:
            dir_ = posixpath.join(base_dir, *prefix) if prefix else base_dir
            if path_attr:
                cands = [posixpath.normpath(posixpath.join(parent, path_attr))]
            else:
                cands = [posixpath.join(dir_, f"{name}.rs"), posixpath.join(dir_, name, "mod.rs")]
            for c in cands:
                c = posixpath.normpath(c)
                if exists(c):
                    stack.append(c)
                    break
    return seen


def crate_roots(crate_dir: str, manifest: dict, paths: set[str]) -> set[str]:
    def j(*parts):
        return posixpath.normpath(posixpath.join(crate_dir, *parts)) if crate_dir else posixpath.normpath(posixpath.join(*parts))

    roots = set()
    for rel in ("src/lib.rs", "src/main.rs", "build.rs"):
        if j(rel) in paths:
            roots.add(j(rel))
    for sub in ("src/bin", "tests", "benches", "examples"):
        d = j(sub) + "/"
        for p in paths:
            if p.startswith(d) and p.endswith(".rs"):
                rest = p[len(d):]
                if "/" not in rest or (rest.count("/") == 1 and rest.endswith("/main.rs")):
                    roots.add(p)
    lib = manifest.get("lib", {})
    if isinstance(lib, dict) and "path" in lib:
        roots.add(j(str(lib["path"])))
    for key in ("bin", "test", "bench", "example"):
        for t in manifest.get(key, []) or []:
            if isinstance(t, dict) and "path" in t:
                roots.add(j(str(t["path"])))
    return roots


# ---------------------------------------------------------------------------
# Cargo.toml
# ---------------------------------------------------------------------------

DEP_TABLES = ("dependencies", "dev-dependencies", "build-dependencies")


def iter_dep_tables(doc):
    for t in DEP_TABLES:
        if t in doc:
            yield doc[t]
    if "target" in doc:
        for _, tgt in doc["target"].items():
            for t in DEP_TABLES:
                if t in tgt:
                    yield tgt[t]
    if "workspace" in doc and "dependencies" in doc["workspace"]:
        yield doc["workspace"]["dependencies"]


def rewrite_manifests(manifests: dict[str, tuple], off: set[str], report: Report):
    """Remove the off features and every reference to them from Cargo.toml files."""
    pkg_names = {}
    for rel, (doc, _) in manifests.items():
        pkg = doc.get("package", {})
        if "name" in pkg:
            pkg_names[str(pkg["name"])] = rel

    def dep_package(doc, key: str) -> str:
        for table in iter_dep_tables(doc):
            if key in table:
                v = table[key]
                if hasattr(v, "get") and v.get("package"):
                    return str(v["package"])
                return key
        return key

    implied_before: dict[str, list[str]] = {}
    for rel, (doc, _) in manifests.items():
        feats = doc.get("features")
        if feats is not None:
            for name in list(feats.keys()):
                if name in off:
                    del feats[name]
                    report.add({"action": "cargo_remove_feature", "file": rel, "feature": name})
            for name, items in feats.items():
                removed = []
                for item in list(items):
                    s = str(item)
                    feat = s
                    if "/" in s:
                        dep, feat = s.split("/", 1)
                        dep = dep.rstrip("?")
                        if dep.startswith("dep:") or dep_package(doc, dep) not in pkg_names:
                            continue
                    elif s.startswith("dep:"):
                        continue
                    if feat in off:
                        removed.append(s)
                if removed:
                    for s in removed:
                        idx = [str(x) for x in items].index(s)
                        del items[idx]
                    implied_before.setdefault(rel, []).append(name)
                    report.add({"action": "cargo_drop_reference", "file": rel, "feature": name,
                                "removed": removed})
        for table in iter_dep_tables(doc):
            for key, v in table.items():
                if not hasattr(v, "get") or "features" not in v:
                    continue
                if dep_package(doc, key) not in pkg_names:
                    continue
                arr = v["features"]
                removed = [str(x) for x in arr if str(x) in off]
                for s in removed:
                    del arr[[str(x) for x in arr].index(s)]
                if removed:
                    report.add({"action": "cargo_drop_dep_feature", "file": rel, "dependency": key,
                                "removed": removed})
    return implied_before


def orphaned_optional_deps(doc) -> list[str]:
    feats = doc.get("features", {})
    refs = set()
    for items in feats.values():
        for item in items:
            s = str(item)
            if s.startswith("dep:"):
                refs.add(s[4:])
            elif "/" in s:
                refs.add(s.split("/", 1)[0].rstrip("?"))
            else:
                refs.add(s)
    out = []
    for table in iter_dep_tables(doc):
        for key, v in table.items():
            if hasattr(v, "get") and v.get("optional") and key not in refs:
                out.append(key)
    return out


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------

SKIP_DIRS = {"target", "node_modules"}


def relevant(path: str) -> bool:
    if SKIP_DIRS.intersection(path.split("/")):
        return False
    return path.endswith(".rs") or posixpath.basename(path) == "Cargo.toml"


def strip_tree(paths: set[str], read, off: set[str], report: Report,
               content_id=None, cache: dict | None = None) -> dict[str, bytes | None]:
    """Compute the stripped tree. `paths` holds every relevant (.rs / Cargo.toml) path.

    Returns {path: new_bytes} for rewritten files and {path: None} for deleted ones.

    `content_id(path)` and `cache` are an optional speed-up for stripping many
    similar trees (git history): give each file a stable id for its content
    (e.g. its blob id) and per-file work is reused across calls. The result is
    the same with or without them.
    """
    use_cache = content_id is not None and cache is not None

    def memo(key, compute):
        if not use_cache:
            return compute()
        hit = cache.get(key, _MISSING)
        if hit is _MISSING:
            hit = cache[key] = compute()
        return hit

    def cid(p):
        return content_id(p) if use_cache else None

    def replay(records, path):
        for rec in records:
            report.add({**rec, "file": path} if "file" in rec else rec)

    manifest_paths = sorted(p for p in paths if posixpath.basename(p) == "Cargo.toml")

    def parse_plain(p):
        try:
            return tomlkit.parse(read(p).decode()).unwrap(), None
        except Exception as e:  # malformed manifest (old history): leave it alone
            return None, f"unparseable Cargo.toml: {e}"

    plain = {}
    for p in manifest_paths:
        doc, err = memo(("manifest", cid(p)), lambda p=p: parse_plain(p))
        if doc is None:
            report.add({"action": "flag", "file": p, "reason": err})
        else:
            plain[p] = doc
    crate_dirs = sorted({posixpath.dirname(p) for p, doc in plain.items() if "package" in doc},
                        key=lambda d: -len(d))

    def owning_crate(path: str) -> str | None:
        for d in crate_dirs:  # longest first
            if d == "" or path.startswith(d + "/"):
                return d
        return None

    needles = [name.encode() for name in off]

    def rewrite_one(f):
        src = read(f)
        if not any(n in src for n in needles):
            return None, False, []
        local = Report()
        try:
            out, delete = rewrite_source(PurePosixPath(f), f, src, off, local)
        except Exception as e:
            return None, False, [{"action": "flag", "file": f, "reason": f"rewrite failed, left unchanged: {e!r}"}]
        records = local.records + local.flags
        if delete:
            return None, True, records
        return (out if out != src else None), False, records

    changes: dict[str, bytes | None] = {}
    for f in sorted(p for p in paths if p.endswith(".rs")):
        if owning_crate(f) is None:
            continue
        out, delete, records = memo(("rewrite", cid(f)), lambda f=f: rewrite_one(f))
        replay(records, f)
        if delete:
            changes[f] = None
        elif out is not None:
            changes[f] = out

    def exists_before(p):
        return p in paths

    def exists_after(p):
        return p in paths and changes.get(p, b"") is not None

    def decls_before(p):
        return memo(("decls", cid(p)), lambda: mod_decls(read(p)))

    def decls_after(p):
        return mod_decls(changes[p]) if p in changes else decls_before(p)

    rs_paths = sorted(p for p in paths if p.endswith(".rs"))
    for p, doc in plain.items():
        if "package" not in doc:
            continue
        crate = posixpath.dirname(p)
        roots = crate_roots(crate, doc, paths)
        prefix = crate + "/" if crate else ""

        def reach(roots=roots):
            before = module_files(roots, decls_before, exists_before)
            after = module_files({r for r in roots if exists_after(r)}, decls_after, exists_after)
            return sorted(before - after)

        if use_cache:
            h = hashlib.sha1(repr(sorted(roots)).encode())
            for f in rs_paths:
                if f.startswith(prefix):
                    after = changes.get(f, 0) if f in changes else 0
                    h.update(f.encode() + b"\0" + str(cid(f)).encode() + b"\0" +
                             (b"del" if after is None else hashlib.sha1(after).digest() if after else b"="))
            gone = memo(("reach", crate, h.digest()), reach)
        else:
            gone = reach()
        for f in gone:
            if changes.get(f, b"") is None:
                continue
            changes[f] = None
            report.add({"action": "delete_file", "file": f, "lines": read(f).count(b"\n") + 1,
                        "reason": "module only declared by deleted cfg-gated `mod`"})

    def manifests_phase():
        manifests = {}
        for p in plain:
            text = read(p).decode()
            manifests[p] = (tomlkit.parse(text), text)
        local = Report()
        implied = rewrite_manifests(manifests, off, local)
        out = {}
        for rel, (doc, text) in manifests.items():
            for dep in orphaned_optional_deps(doc):
                local.add({"action": "flag", "file": rel, "dependency": dep,
                           "reason": "optional dependency no longer enabled by any feature"})
            for feat in implied.get(rel, []):
                local.add({"action": "flag", "file": rel, "feature": feat,
                           "reason": "feature previously enabled an off feature; combination untested"})
            new = tomlkit.dumps(doc)
            if new != text:
                out[rel] = new.encode()
        return out, local.records + local.flags

    manifest_changes, records = memo(("manifests", tuple((p, cid(p)) for p in sorted(plain))), manifests_phase)
    for rec in records:
        report.add(rec)
    changes.update(manifest_changes)
    return changes


_MISSING = object()


def run(root: Path, off: set[str], dry_run: bool) -> Report:
    root = root.resolve()
    report = Report()
    paths = set()
    for p in root.rglob("*"):
        rel = p.relative_to(root).as_posix()
        if p.is_file() and relevant(rel):
            paths.add(rel)
    changes = strip_tree(paths, lambda rel: (root / rel).read_bytes(), off, report)
    if not dry_run:
        for rel, out in changes.items():
            target = root / rel
            if out is None:
                target.unlink()
            else:
                target.write_bytes(out)
    return report


def tool_version() -> str:
    """Version string that pins the exact stripping logic: release + source hash."""
    digest = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()[:12]
    return f"{__version__}+{digest}"


LOG_HEADER = (
    "cfgstrip log. Records where code gated on the off features was removed, "
    "unwrapped or rewritten. By design it never contains the removed text."
)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("root", type=Path)
    ap.add_argument("--off", action="append", default=None,
                    help=f"feature to force off (repeatable; default: {', '.join(DEFAULT_OFF)})")
    ap.add_argument("--log", type=Path, default=Path("cfgstrip-log.jsonl"))
    ap.add_argument("--dry-run", action="store_true", help="compute and log, change nothing")
    args = ap.parse_args(argv)
    off = set(args.off or DEFAULT_OFF)

    report = run(args.root, off, args.dry_run)
    write_log(args.log, report, off, args.dry_run)
    print_summary(report, off, args.dry_run, args.log)
    return 0


def write_log(path: Path, report: Report, off: set[str], dry_run: bool, extra: dict | None = None) -> None:
    with path.open("w") as fh:
        fh.write(json.dumps({"header": LOG_HEADER, "off": sorted(off), "dry_run": dry_run,
                             "tool": tool_version(), **(extra or {})}) + "\n")
        for rec in report.records + report.flags:
            fh.write(json.dumps(rec, default=str) + "\n")


def print_summary(report: Report, off: set[str], dry_run: bool, log: Path) -> None:
    counts: dict[str, int] = {}
    lines_removed = 0
    for rec in report.records:
        counts[rec["action"]] = counts.get(rec["action"], 0) + 1
        if rec["action"] in ("delete", "delete_file"):
            lines_removed += rec.get("lines", 0)
    print(f"{'dry run: ' if dry_run else ''}off = {', '.join(sorted(off))}")
    for k in sorted(counts):
        print(f"  {k:24} {counts[k]}")
    print(f"  {'lines removed':24} {lines_removed}")
    print(f"  {'flags for review':24} {len(report.flags)}")
    print(f"log: {log}")


if __name__ == "__main__":
    sys.exit(main())
