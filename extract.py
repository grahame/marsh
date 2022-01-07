#!/usr/bin/env python3

import json
import sys
import re


def main():
    if len(sys.argv) != 3:
        print("Usage: python extract.py <bundle.js> <wordle.json>")
        sys.exit(1)

    with open(sys.argv[1]) as fd:
        data = fd.read()

    arrays = []

    while True:
        m = re.search(r"\[\"[a-z][a-z][a-z][a-z][a-z]\",", data)
        if not m:
            break
        s, _ = m.span()
        e2 = s + data[s:].find("]") + 1
        extracted = json.loads(data[s:e2])
        valid = all(type(t) is str and len(t) == 5 for t in extracted)
        if valid:
            arrays.append(extracted)
        else:
            print("skipped invalid list in .js file:", extracted)
        data = data[e2:]

    print(len(arrays))
    assert len(arrays) == 2
    arrays.sort(key=lambda x: len(x))
    # there are two lists. the first list contains the possible
    # solution space, ~2500 words. the second list contains a
    # bunch of obscure words which wordle will accept as guesses,
    # but which will never be answers.
    #
    # citation: https://www.nytimes.com/2022/01/03/technology/wordle-word-game-creator.html
    words = arrays[0]
    assert len(set(words)) == len(words)
    print("extracted {} words".format(len(words)))
    with open(sys.argv[2], "w") as fd:
        json.dump(sorted(words), fd)


if __name__ == "__main__":
    main()
