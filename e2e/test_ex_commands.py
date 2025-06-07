from .helpers import run_commands


def test_write_command():
    result = run_commands(['i', 'written', '\x1b', ':w\r', ':q\r'], initial_content='')
    assert result.strip() == 'written'


def test_exit_with_x():
    result = run_commands(['i', 'done', '\x1b', ':x\r'], initial_content='', exit_cmd=None)
    assert result.strip() == 'done'


def test_substitute_range():
    content = 'abc\ndef\nabc\nabc\n'
    result = run_commands([':1,3s/^abc/cde/\r'], initial_content=content)
    assert result.splitlines() == ['cde', 'def', 'cde', 'abc']


def test_substitute_range_undo():
    content = 'abc\ndef\nabc\nabc\n'
    result = run_commands([':1,3s/^abc/cde/\r', 'u'], initial_content=content)
    assert result.splitlines() == ['abc', 'def', 'abc', 'abc']


def test_substitute_range_repeat():
    content = 'abc\ndef\nabc\nabc\n'
    result = run_commands([':1,3s/^abc/cde/\r', '.'], initial_content=content)
    assert result.splitlines() == ['cde', 'def', 'cde', 'abc']


def test_delete_line():
    result = run_commands([':1d\r'], initial_content='a\nb\n')
    assert result.splitlines() == ['b']


def test_delete_line_undo():
    result = run_commands([':1d\r', 'u'], initial_content='a\nb\n')
    assert result.splitlines() == ['a', 'b']


def test_delete_line_repeat():
    result = run_commands([':1d\r', '.'], initial_content='a\nb\n')
    # Ex commands cannot be repeated with '.'
    assert result.splitlines() == ['b']


def test_move_line():
    result = run_commands([':1m5\r'], initial_content='1\n2\n3\n4\n5\n6\n')
    assert result.splitlines() == ['2', '3', '4', '5', '1', '6']


def test_move_line_undo():
    result = run_commands([':1m5\r', 'u'], initial_content='1\n2\n3\n4\n5\n6\n')
    assert result.splitlines() == ['1', '2', '3', '4', '5', '6']


def test_move_line_repeat():
    result = run_commands([':1m5\r', '.'], initial_content='1\n2\n3\n4\n5\n6\n')
    # Ex commands cannot be repeated with '.'
    assert result.splitlines() == ['2', '3', '4', '5', '1', '6']


def test_copy_line():
    result = run_commands([':1co5\r'], initial_content='1\n2\n3\n4\n5\n6\n')
    assert result.splitlines() == ['1', '2', '3', '4', '5', '1', '6']


def test_copy_line_undo():
    result = run_commands([':1co5\r', 'u'], initial_content='1\n2\n3\n4\n5\n6\n')
    assert result.splitlines() == ['1', '2', '3', '4', '5', '6']


def test_copy_line_repeat():
    result = run_commands([':1co5\r', '.'], initial_content='1\n2\n3\n4\n5\n6\n')
    # Ex commands cannot be repeated with '.'
    assert result.splitlines() == ['1', '2', '3', '4', '5', '1', '6']


def test_move_line_to_top():
    result = run_commands([':2m0\r'], initial_content='1\n2\n3\n')
    assert result.splitlines() == ['2', '1', '3']


def test_move_line_to_top_undo():
    result = run_commands([':2m0\r', 'u'], initial_content='1\n2\n3\n')
    assert result.splitlines() == ['1', '2', '3']


def test_move_line_to_top_repeat():
    result = run_commands([':2m0\r', '.'], initial_content='1\n2\n3\n')
    assert result.splitlines() == ['2', '1', '3']


def test_copy_line_to_top():
    result = run_commands([':3co0\r'], initial_content='1\n2\n3\n')
    assert result.splitlines() == ['3', '1', '2', '3']


def test_copy_line_to_top_undo():
    result = run_commands([':3co0\r', 'u'], initial_content='1\n2\n3\n')
    assert result.splitlines() == ['1', '2', '3']


def test_copy_line_to_top_repeat():
    result = run_commands([':3co0\r', '.'], initial_content='1\n2\n3\n')
    assert result.splitlines() == ['3', '1', '2', '3']


def test_move_line_out_of_range():
    result = run_commands([':1m100\r'], initial_content='1\n2\n3\n')
    assert result.splitlines() == ['2', '3', '1']


def test_move_line_out_of_range_undo():
    result = run_commands([':1m100\r', 'u'], initial_content='1\n2\n3\n')
    assert result.splitlines() == ['1', '2', '3']


def test_move_line_out_of_range_repeat():
    result = run_commands([':1m100\r', '.'], initial_content='1\n2\n3\n')
    assert result.splitlines() == ['2', '3', '1']


def test_copy_reverse_range():
    result = run_commands([':3,1co$\r'], initial_content='1\n2\n3\n4\n')
    assert result.splitlines() == ['1', '2', '3', '4', '1', '2', '3']


def test_copy_reverse_range_undo():
    result = run_commands([':3,1co$\r', 'u'], initial_content='1\n2\n3\n4\n')
    assert result.splitlines() == ['1', '2', '3', '4']


def test_copy_reverse_range_repeat():
    result = run_commands([':3,1co$\r', '.'], initial_content='1\n2\n3\n4\n')
    assert result.splitlines() == ['1', '2', '3', '4', '1', '2', '3']


def test_move_reverse_range():
    result = run_commands([':3,1m$\r'], initial_content='1\n2\n3\n4\n')
    assert result.splitlines() == ['4', '1', '2', '3']


def test_move_reverse_range_undo():
    result = run_commands([':3,1m$\r', 'u'], initial_content='1\n2\n3\n4\n')
    assert result.splitlines() == ['1', '2', '3', '4']


def test_move_reverse_range_repeat():
    result = run_commands([':3,1m$\r', '.'], initial_content='1\n2\n3\n4\n')
    assert result.splitlines() == ['4', '1', '2', '3']
