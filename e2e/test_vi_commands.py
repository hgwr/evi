from .helpers import run_commands


def test_append_after_cursor():
    result = run_commands(['a', 'b', '\x1b'], initial_content='ac\n')
    assert result.strip() == 'abc'


def test_open_line_above():
    result = run_commands(['O', 'first', '\x1b'], initial_content='second\n')
    assert result.splitlines() == ['first', 'second']


def test_delete_char():
    result = run_commands(['l', 'x'], initial_content='abc\n')
    assert result.strip() == 'ac'


def test_delete_word():
    result = run_commands(['d', 'w'], initial_content='foo bar\n')
    assert result.strip() == 'bar'


def test_search_forward_delete_line():
    result = run_commands(['/bar\r', 'dd'], initial_content='foo\nbar\nbaz\n')
    assert result.splitlines() == ['foo', 'baz']


def test_search_backward_delete_line():
    result = run_commands(['j', '?foo\r', 'dd'], initial_content='foo\nbar\nfoo\n')
    assert result.splitlines() == ['bar', 'foo']


def test_undo():
    result = run_commands(['x', 'u'], initial_content='hello\n')
    assert result.strip() == 'hello'


def test_repeat_command():
    result = run_commands(['x', '.', '.'], initial_content='abc\n')
    assert result.strip() == ''


def test_undo_then_repeat():
    result = run_commands(['x', 'u', '.'], initial_content='abc\n')
    assert result.strip() == 'bc'


def test_write_quit_ZZ():
    result = run_commands(['i', 'done', '\x1b', 'ZZ'], exit_cmd=None)
    assert result.strip() == 'done'


def test_cursor_motion_zero_dollar():
    result = run_commands(['$', 'x', 'u', '0', 'x'], initial_content='abc\n')
    assert result.strip() == 'bc'


def test_word_motion_w_b():
    result = run_commands(['w', 'x', 'b', 'x'], initial_content='one two\n')
    assert result.strip() == 'ne wo'
