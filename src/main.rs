use rayon::prelude::*;
use std::fs::File;
use std::env;

enum Mark {
    Yellow = 1,
    Green = 2,
    Excluded = 3,
}

#[derive(Debug, Copy, Clone)]
struct Word {
    chars: [u8; 5]
}

#[derive(Debug)]
struct CharCount {
    count: [u8; 26]
}

#[derive(Debug)]
struct Score {
    marks: [u8; 5]
}

fn read_words(path: &str) -> Vec<Word> {
    let file = File::open(path).unwrap();
    let w: Vec<String> = serde_json::from_reader(file).unwrap();
    w.iter().map(|w: &String| encode_word(w)).collect()
}

fn encode_word(word: &str) -> Word {
    let mut w = Word { chars: [0; 5] };
    for (i, c) in word.chars().enumerate() {
        w.chars[i] = (c as usize - 'a' as usize) as u8;
    }
    w
}

fn decode_word(word: &Word) -> String {
    let mut w = String::new();
    for c in word.chars.iter() {
        w.push((*c as usize + 'a' as usize) as u8 as char);
    }
    w
}

fn count_word(word: &Word) -> CharCount {
    let mut cf = CharCount { count: [0; 26] };
    for c in word.chars.iter() {
        cf.count[*c as usize] += 1;
    }
    cf
}

fn mark_guess(word: &Word, guess: &Word) -> Score {
    let mut score = Score {
        marks: [0; 5],
    };
    let mut cf = count_word(word);

    // mark off excluded (totally non-present) letters
    for (i, g_c) in guess.chars.iter().enumerate() {
        if cf.count[*g_c as usize] == 0 {
            score.marks[i] = Mark::Excluded as u8;
        }
    }

    // find exact matches and score them green
    for (i, (g, w)) in guess.chars.iter().zip(word.chars.iter()).enumerate() {
        if g == w {
           score.marks[i] = Mark::Green as u8;
            cf.count[*g as usize] -= 1;
        }
    }

    // find inexact matches and score them yellow
    for (i, g) in guess.chars.iter().enumerate() {
        if score.marks[i] != Mark::Excluded as u8 {
            continue;
        }
        if cf.count[*g as usize] > 0 {
            score.marks[i] = Mark::Yellow as u8;
            cf.count[*g as usize] -= 1;
        }
    }
    score
}

fn next_words(words: &[Word], word: &Word, guess: &Word) -> Vec<Word> {
    // given a word and a guess, determine our next guess
    let mut res = Vec::new();
    let mut ex = CharCount { count: [0; 26] };
    let score = mark_guess(word, guess);

    // determine excluded characters
    for (i, m) in score.marks.iter().enumerate() {
        if *m == Mark::Excluded as u8 {
            ex.count[guess.chars[i] as usize] += 1;
        }
    }

    // constrained search for possible next guesses
    for w in words.iter() {
        if w.chars == guess.chars {
            continue;
        }

        let mut possible = true;
        let mut cf = count_word(word);
        // does the word contain a character that is excluded?
        for c in w.chars.iter() {
            if ex.count[*c as usize] > 0 {
                possible = false;
                break;
            }
        }
        if !possible {
            continue;
        }
        // check green letters match
        for ((w_c, g_c), m) in w.chars.iter().zip(guess.chars.iter()).zip(score.marks.iter()) {
            if *m == Mark::Green as u8 {
                if *w_c != *g_c {
                    possible = false;
                    break;
                }
                cf.count[*g_c as usize] -= 1;
            }
        }
        if !possible {
            continue;
        }
        // check yellow letters possible
        for (g_c, m) in guess.chars.iter().zip(score.marks.iter()) {
            if *m == Mark::Yellow as u8 {
                if cf.count[*g_c as usize] == 0 {
                    possible = false;
                    break;
                }
                cf.count[*g_c as usize] -= 1;
            }
        }
        if !possible {
            continue;
        }

        res.push(*w);
    }
    res
}

fn solve(words: &[Word], word: &Word, guess: &Word) -> (u32, Word) {
    let mut i: u32 = 0;
    let mut nw: Vec<Word> = words.to_vec();
    let mut next_guess: Word = *guess;

    loop {
        println!("{}: guess: {}", i, decode_word(&next_guess));
        if next_guess.chars == word.chars {
            break;
        }
        nw = next_words(&nw, word, &next_guess);
        next_guess = rankguesses(&nw, &nw)[0].0;
        i += 1;
    }

    (i, next_guess)
}

fn determine_candidates(words: &[Word]) -> Vec<Word> {
    let mut candidates = Vec::new();
    for word in words.iter() {
        let cf = count_word(word);
        let mut valid = true;
        for c in cf.count {
            if c > 1 {
                valid = false;
            }
        }
        if valid {
            candidates.push(*word);
        }
    }
    candidates
}

fn rankguesses(candidates: &[Word], words: &[Word]) -> Vec<(Word, f64)> {
    let mut guesses: Vec<(Word, f64)> = candidates.par_iter().map(|starting_guess: &Word| {
        let ratios: Vec<f64> = words.iter().map(|w| {
            let nw = next_words(words, w, starting_guess);
            (nw.len() as f64) / (words.len() as f64)
        }).collect();

        let arat = ratios.iter().sum::<f64>() / (ratios.len() as f64);

        (*starting_guess, arat)
    }).collect();
    guesses.sort_by(|&(_, r), (_, s)| r.partial_cmp(s).unwrap());
    guesses
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 4 {
        let words = read_words(args[1].as_str());
        let word = args[2].as_str();
        let guess = args[3].as_str();
        println!("solve mode: word={}, guess={}", word, guess);
        solve(&words, &encode_word(word), &encode_word(guess));
    } else if args.len() == 2 {
        let words = read_words(args[1].as_str());
        let candidates = determine_candidates(&words);
        eprintln!("candidates: {} of {} total words", candidates.len(), words.len());
        let ranked = rankguesses(&candidates, &words);
        eprintln!("best word is: {}", decode_word(&ranked[0].0));
        println!("word,average_ratio");
        for (w, r) in ranked.iter() {
            println!("{},{}", decode_word(w), r);
        }
    } else {
        println!("usage: {} src/wordle.json  # determine best first guess", args[0]);
        println!("usage: {} src/wordle.json [word] [guess]  # solve puzzle", args[0]);
    }
}
