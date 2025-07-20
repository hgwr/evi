# :e! and :r commands are not implemented yet
# import os
# import tempfile
# from .helpers import run_commands


# def test_reload_file():
#     result = run_commands(['Achanged', '\x1b', ':e!\r'], initial_content='orig\n')
#     assert result.splitlines() == ['orig']


# def test_read_file():
#     fd, path = tempfile.mkstemp()
#     try:
#         with os.fdopen(fd, 'w') as f:
#             f.write('a\nb\n')
#         result = run_commands([f':r {path}\r'], initial_content='orig\n')
#     finally:
#         os.unlink(path)
#     assert result.splitlines() == ['orig', 'a', 'b']

