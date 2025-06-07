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


def test_delete_line():
    result = run_commands([':1d\r'], initial_content='a\nb\n')
    assert result.splitlines() == ['b']


def test_move_line():
    result = run_commands([':1m5\r'], initial_content='1\n2\n3\n4\n5\n6\n')
    assert result.splitlines() == ['2', '3', '4', '5', '1', '6']


def test_copy_line():
    result = run_commands([':1co5\r'], initial_content='1\n2\n3\n4\n5\n6\n')
    assert result.splitlines() == ['1', '2', '3', '4', '5', '1', '6']


def test_move_line_to_top():
    result = run_commands([':2m0\r'], initial_content='1\n2\n3\n')
    assert result.splitlines() == ['2', '1', '3']


def test_copy_line_to_top():
    result = run_commands([':3co0\r'], initial_content='1\n2\n3\n')
    assert result.splitlines() == ['3', '1', '2', '3']


def test_move_line_out_of_range():
    result = run_commands([':1m100\r'], initial_content='1\n2\n3\n')
    assert result.splitlines() == ['2', '3', '1']
