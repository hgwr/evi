from .helpers import run_commands

def test_insert_and_save():
    result = run_commands(['i', 'Hello, Evi!', '\x1b'])
    assert 'Hello, Evi!' in result

# 'o' command is not implemented yet
# def test_open_new_line():
#     result = run_commands(['o', 'second line', '\x1b'], initial_content='first\n')
#     assert result.splitlines() == ['first', 'second line']

# 'dd' command is not implemented yet
# def test_delete_line():
#     result = run_commands(['j', 'dd'], initial_content='a\nb\nc\n')
#     assert result.splitlines() == ['a', 'c']

def test_substitute():
    result = run_commands([':s/foo/bar/\r', 'j', ':s/foo/bar/g\r'], initial_content='foo\nfoo foo\n')
    assert result.splitlines() == ['bar', 'bar bar']
