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

To determine the best starting guess:

```sh
cargo run --release words.json
```

This will take a while to run. The code is written in rust and makes use of rayon for parallelism.
On my laptop, it takes less than ten seconds using all 8 cores.  The output of this command is a
sorted CSV file, from best initial guess to worst initial guess.

To apply the best guess algorith in solving a hypothetical puzzle:

```sh
# solution: flood
# starting guess: arose
cargo run --release words.json flood arose
```

## Results

tl;dr? Here are the best ten initial guesses:

```csv
word,average_ratio
raise,0.03770974347970099
irate,0.038317853794158675
arise,0.038711940625743575
arose,0.03946279546016459
alter,0.03976918304419014
later,0.03999346920496897
alert,0.04010150721419608
stare,0.04029911041241997
snare,0.04127275865446939
saner,0.04132239269670485
```

So, `raise` is the best first guess, at least at the moment.

... and lucky last, the worst guess:

```csv
jumbo,0.45019681017312085
```

The worlde word list does seem to change from time to time, so the results above may change.
They're current as of 2022-01-08T00:39:43.
