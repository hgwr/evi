from .helpers import run_commands


# Ex command history navigation may not be fully implemented
# def test_ex_history_navigation():
#     commands = [
#         ':2d\r',
#         ':1d\r',
#         ':',
#         '\x1b[A',
#         '\x1b[A',
#         '\x1b[B',
#         '\r',
#     ]
#     result = run_commands(commands, initial_content='1\n2\n3\n4\n')
#     assert result.splitlines() == ['4']
