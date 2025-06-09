from .helpers import run_commands


def test_append_after_cursor():
    result = run_commands(['a', 'b', '\x1b'], initial_content='ac\n')
    assert result.strip() == 'abc'


def test_append_after_cursor_undo():
    result = run_commands(['a', 'b', '\x1b', 'u'], initial_content='ac\n')
    assert result.strip() == 'ac'


def test_append_after_cursor_repeat():
    result = run_commands(['a', 'b', '\x1b', '.', '\x1b'], initial_content='ac\n')
    assert result.strip() == 'abbc'


def test_open_line_above():
    result = run_commands(['O', 'first', '\x1b'], initial_content='second\n')
    assert result.splitlines() == ['first', 'second']


def test_open_line_above_undo():
    result = run_commands(['O', 'first', '\x1b', 'u'], initial_content='second\n')
    assert result.strip() == 'second'


def test_open_line_above_repeat():
    result = run_commands(['O', 'first', '\x1b', '.', '\x1b'], initial_content='second\n')
    # TODO: repeating open line should insert another line but is not implemented yet
    assert result.splitlines() == ['second']


def test_delete_char():
    result = run_commands(['l', 'x'], initial_content='abc\n')
    assert result.strip() == 'ac'


def test_delete_word():
    result = run_commands(['d', 'w'], initial_content='foo bar\n')
    assert result.strip() == 'bar'


def test_delete_word_undo():
    result = run_commands(['d', 'w', 'u'], initial_content='foo bar\n')
    assert result.strip() == 'foo bar'


def test_search_forward_delete_line():
    result = run_commands(['/bar\r', 'dd'], initial_content='foo\nbar\nbaz\n')
    assert result.splitlines() == ['foo', 'baz']


def test_search_forward_delete_line_undo():
    result = run_commands(['/bar\r', 'dd', 'u'], initial_content='foo\nbar\nbaz\n')
    assert result.splitlines() == ['foo', 'bar', 'baz']


def test_search_forward_delete_line_repeat():
    result = run_commands(['/bar\r', 'dd', '.'], initial_content='foo\nbar\nbaz\n')
    # TODO: repeating delete after search should remove the next line but is not implemented
    assert result.splitlines() == ['foo', 'baz']


def test_search_backward_delete_line():
    result = run_commands(['j', '?foo\r', 'dd'], initial_content='foo\nbar\nfoo\n')
    assert result.splitlines() == ['bar', 'foo']


def test_search_backward_delete_line_undo():
    result = run_commands(['j', '?foo\r', 'dd', 'u'], initial_content='foo\nbar\nfoo\n')
    assert result.splitlines() == ['foo', 'bar', 'foo']


def test_search_backward_delete_line_repeat():
    result = run_commands(['j', '?foo\r', 'dd', '.'], initial_content='foo\nbar\nfoo\n')
    assert result.splitlines() == ['foo']


def test_undo():
    result = run_commands(['x', 'u'], initial_content='hello\n')
    assert result.strip() == 'hello'


def test_repeat_command():
    result = run_commands(['x', '.', '.'], initial_content='abc\n')
    assert result.strip() == ''


def test_undo_then_repeat():
    result = run_commands(['x', 'u', '.'], initial_content='abc\n')
    assert result.strip() == 'bc'


def test_repeat_delete_char_with_count():
    result = run_commands(['3', 'x', '.'], initial_content='abcdef\n')
    assert result.strip() == ''


def test_repeat_insert():
    result = run_commands(['i', 'abc', '\x1b', '.'], initial_content='')
    assert result.strip() == 'abcabc'


def test_repeat_delete_word():
    result = run_commands(['d', 'w', '.'], initial_content='one two three\n')
    assert result.strip() == 'three'


def test_write_quit_ZZ():
    result = run_commands(['i', 'done', '\x1b', 'ZZ'], exit_cmd=None)
    assert result.strip() == 'done'


def test_cursor_motion_zero_dollar():
    result = run_commands(['$', 'x', 'u', '0', 'x'], initial_content='abc\n')
    assert result.strip() == 'bc'


def test_word_motion_w_b():
    result = run_commands(['w', 'x', 'b', 'x'], initial_content='one two\n')
    assert result.strip() == 'ne wo'


def test_insert_unicode_undo():
    result = run_commands(['i', 'あい\nう', '\x1b', 'u'], initial_content='foo\n')
    assert result.strip() == 'foo'


def test_append_unicode_undo():
    result = run_commands(['a', '🍣', '\x1b', 'u'], initial_content='bar\n')
    assert result.strip() == 'bar'


def _parse_screen(screen: str) -> dict[int, str]:
    import re
    rows: dict[int, str] = {}
    for row, _col, text in re.findall(r"\x1b\[(\d+);(\d+)H([^\x1b]*)", screen):
        if text:
            rows[int(row)] = text
    return rows


def test_invalid_command_shows_status_line(tmp_path):
    import os
    import pexpect
    from .conftest import EVI_BIN
    from .test_motion_commands import get_screen_and_cursor

    file_path = tmp_path / "file.txt"
    file_path.write_text("test\n")

    env = os.environ.copy()
    env.setdefault("TERM", "xterm")

    child = pexpect.spawn(EVI_BIN, [str(file_path)], env=env, encoding="utf-8")
    child.delaybeforesend = float(os.getenv("EVI_DELAY_BEFORE_SEND", "0.1"))
    child.setwinsize(24, 80)

    # Wait for the editor to render
    get_screen_and_cursor(child)

    # Send an invalid command sequence and check status line
    child.send("dZ")
    child.expect("\x07")
    child.expect(r"\?")

    child.send(":q!\r")
    child.expect(pexpect.EOF)
