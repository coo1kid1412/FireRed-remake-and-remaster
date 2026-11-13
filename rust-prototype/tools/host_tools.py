"""Host compiler for the Rust prototype's data import and checks."""

import os

MINGW = 'C:/msys64/mingw64/bin'
if os.name == 'nt':
    CC = f'{MINGW}/gcc.exe'
    ENV = dict(os.environ, PATH=MINGW + os.pathsep + os.environ.get('PATH', ''))
else:
    CC = os.environ.get('CC', 'cc')
    ENV = os.environ.copy()
