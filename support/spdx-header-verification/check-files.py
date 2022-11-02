#!/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Based on: https://github.com/enarx/spdx

import os
import re

SLUG = re.compile('[a-zA-Z0-9.-]+')
SPDX = re.compile(f'SPDX-License-Identifier:\s+({SLUG.pattern})')


class Language:
    def __init__(self, *comments, shebang=False):
        assert(isinstance(shebang, bool))
        self.__shebang = shebang

        self.__match = []
        for comment in comments:
            (init, fini) = (comment, '')
            if isinstance(comment, tuple):
                (init, fini) = comment

            pattern = f"^{init}\s*{SPDX.pattern}\s*{fini}\s*$"
            self.__match.append(re.compile(pattern))

    def license(self, path):
        "Find the license from the SPDX header."
        with open(path) as f:
            line = f.readline()
            if self.__shebang and line.startswith('#!'):
                line = f.readline()

        for matcher in self.__match:
            match = matcher.match(line)
            if match:
                return match.group(1)

        return None


class Index:
    INTERPRETERS = {
        'python3': 'python',
        'python2': 'python',
        'python': 'python',
        'ruby': 'ruby',
        'lua': 'lua',
    }

    EXTENSIONS = {
        '.py': 'python',
        '.proto': 'protobuf',
        '.rs': 'rust',
        '.yml': 'yaml',
        '.yaml': 'yaml',
        '.json': 'json',
        '.toml': 'toml',
        '.md': 'md',
        '.rb': 'ruby',
        '.c': 'c',
        '.h': 'c',
        '.cpp': 'c++',
        '.hpp': 'c++',
        '.cc': 'c++',
        '.hh': 'c++',
        '.lua': 'lua',
    }

    def __init__(self):
        self.__languages = {
            'python': Language('#+', shebang=True),
            'ruby': Language('#+', shebang=True),
            'c': Language('//+', ('/\\*', '\\*/')),
            'c++': Language('//+', ('/\\*', '\\*/')),
            'rust': Language('//+', '//!', ('/\\*', '\\*/')),
            'protobuf': Language('//+', '//!', ('/\\*', '\\*/')),
            'lua': Language('--+', ('--\[\[', '--\]\]')),
        }

    def language(self, path):
        name = self.EXTENSIONS.get(os.path.splitext(path)[1])
        if name is None:
            interpreter = None
            with open(path, "rb") as f:
                if f.read(2) == bytearray('#!'.encode('ascii')):
                    # assume a text file and retry as text file
                    try:
                        with open(path, "r") as t:
                            interpreter = t.readline().rstrip().rsplit(
                                os.path.sep)[-1]
                    except:
                        pass
            name = self.INTERPRETERS.get(interpreter)
        return self.__languages.get(name)

    def scan(self, root):
        IGNORE_DIRS = {".git", "target"}

        for root, dirs, files in os.walk(root):
            # Ignore the specified directories.
            for dir in IGNORE_DIRS.intersection(dirs):
                dirs.remove(dir)

            for file in files:
                path = os.path.join(root, file)

                # Find the language of the file.
                language = self.language(path)
                if language is None:
                    continue

                # Parse the SPDX header for the language.
                yield (path, language.license(path))


if __name__ == '__main__':
    import sys
    import json

    # Validate the arguments
    licenses = os.getenv('INPUT_LICENSES')
    if licenses is None:
        licenses = sys.argv[1:]
    else:
        licenses = json.loads(licenses)
    for license in licenses:
        if not SLUG.match(license):
            print("Invalid license '%s'!" % license)
            raise SystemExit(1)

    rv = 0
    index = Index()
    for (path, license) in index.scan("."):
        if license not in licenses:
            if license == None:
                print(f"NO SPDX HEADER\t {path}")
            else:
                print(f"{license:16} {path}")
            rv = 1

    raise SystemExit(rv)
