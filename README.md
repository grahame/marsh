# marsh: determine the best wordle starting move

## Overview

This program estimates the best starting move for [wordle](https://www.powerlanguage.co.uk/wordle/).

We determine the starting move which, on average, reduces the number of possible remaining words
in the wordle dictionary by the greatest percentage: narrowing the search space by the greatest
average amount.

This project implements a brute force search to determine the answer.

The algorithm is:

1. Iterate over each word in the wordle dictionary, as a candidate first guess. To speed things up, we exclude words with duplicate letters – a duplicate letter is a waste of a letter.
2. Taking each word in the dictionary as the word of the day, determine the number of possible words after guessing our candidate word. This is done by applying the scoring mechanism used on the wordle site, determining the tiles which would be marked green and yellow, and which characters would be excluded as not present in the word; and using this information, filtering the list of all words. We then calculate the ratio `n_possibilities / n_words`.
3. For each first guess, we average the calculated ratios, giving us an average reduction in the number of possible words for that starting guess.
4. The best starting guess is that with the lowest average ratio.

In addition to the disclaimers in `LICENSE.txt`: this is a bit of fun, written one evening. If you can think
of a better approach, please send me a PR 😊

## Running the code

First, you'll need to download the Javascript file from the wordle website.
Then, extract the wordlist from it:

```sh
python3 extract.py wordle.js words.json
```

### calculate mode

To determine the best starting guess:

```sh
cargo run --release words.json calculate
```

This will take a short while to run. The code is written in rust and makes use of rayon for parallelism.
On my laptop, it takes less than ten seconds using all 8 cores.  The output of this command is a
sorted CSV file, from best initial guess to worst initial guess.

### solver mode

To apply the best guess algorith in solving a hypothetical puzzle:

```sh
# solution: flood
# starting guess: arose
cargo run --release words.json solve flood arose
```

### bot mode

To solve today's puzzle using `marsh`, add arguments after each guess on the wordle site.
Arguments are added in pairs, one for the word guessed, and one for the mark provided
by wordle, encoded with:

- `g`: yellow tile
- `y`: yellow tile
- `x`: character excluded

Note that a black hint tile does not always mean that a character has been excluded. If in
doubt, check the keyboard at the bottom of the wordle screen.

For example:

```sh
# enter raise into wordle
$ cargo run --release words.json bot raise yyxxx
raise: yxxxx
-> bot move: acorn
# enter acorn into wordle, then re-run adding the next two arguments..
```

## Results

tl;dr? Here are the best ten initial guesses:

```csv
word,average_ratio
raise,0.026350078602783357
arise,0.0275271144615128
irate,0.027550252135336916
arose,0.02851867574136189
alter,0.030233849110645705
saner,0.0302916932952062
later,0.030333863571691932
snare,0.030711530118627744
stare,0.030796617048173932
slate,0.03091678367674437
```

So, `raise` is the best first guess, at least at the moment.

... and lucky last, the worst guess:

```csv
jumpy,0.25890758458545743
```

The worlde word list does seem to change from time to time, so the results above may change.
They're current as of 2022-01-08T00:39:43.
