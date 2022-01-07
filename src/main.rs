use rayon::prelude::*;
use std::fs::File;
use std::env;

enum Mark {
    NoScore = 0,
    Excluded = 1,
    Yellow = 2,
    Green = 3,
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
        if score.marks[i] != Mark::NoScore as u8 {
            continue;
        }
        if cf.count[*g as usize] > 0 {
            score.marks[i] = Mark::Yellow as u8;
            cf.count[*g as usize] -= 1;
        }
    }
    score
}

fn word_valid(guess: &Word, candidate: &Word, ex: &CharCount, score: &Score) -> bool {
    let mut candidate_cf = count_word(candidate);
    // does the word contain a character that is excluded?
    for c in candidate.chars.iter() {
        if ex.count[*c as usize] > 0 {
            return false;
        }
    }
    // check green letters match; yellow letters don't
    for ((w_c, g_c), m) in candidate.chars.iter().zip(guess.chars.iter()).zip(score.marks.iter()) {
        if *m == Mark::Green as u8 {
            if *w_c != *g_c {
                return false;
            }
            candidate_cf.count[*g_c as usize] -= 1;
        }
        if *m == Mark::Yellow as u8 {
            if *w_c == *g_c {
                return false;
            }
        }
    }
    // check yellow letters possible
    for (g_c, m) in guess.chars.iter().zip(score.marks.iter()) {
        if *m == Mark::Yellow as u8 {
            if candidate_cf.count[*g_c as usize] == 0 {
                return false;
            }
            candidate_cf.count[*g_c as usize] -= 1;
        }
    }
    true
}

fn next_words(words: &[Word], score: &Score, guess: &Word) -> Vec<Word> {
    // given a word and a guess, determine our next guess
    let mut res = Vec::new();
    let mut ex = CharCount { count: [0; 26] };

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
        if word_valid(guess, w, &ex, &score) {
            res.push(*w);
        }
    }
    res
}

fn solve(words: &[Word], word: &Word, guess: &Word) -> (u32, Word) {
    let mut i: u32 = 0;
    let mut nw: Vec<Word> = words.to_vec();
    let mut next_guess: Word = *guess;

    loop {
        if next_guess.chars == word.chars {
            println!("{}:  done {}", i, decode_word(&next_guess));
            break;
        }
        let l_b = nw.len();
        let score = mark_guess(word, guess);
        nw = next_words(&nw, &score, &next_guess);
        println!("{}: guess {} ({} -> {})", i, decode_word(&next_guess), l_b, nw.len());
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
            let score = mark_guess(w, starting_guess);
            let nw = next_words(words, &score, starting_guess);
            (nw.len() as f64) / (words.len() as f64)
        }).collect();

        let arat = ratios.iter().sum::<f64>() / (ratios.len() as f64);

        (*starting_guess, arat)
    }).collect();
    guesses.sort_by(|&(_, r), (_, s)| r.partial_cmp(s).unwrap());
    guesses
}

fn cli_score(guess: &Word, hint: &String) -> Score {
    let mut score = Score {
        marks: [0; 5],
    };
    for (i, c) in hint.chars().enumerate() {
        if c == 'x' {
            score.marks[i] = Mark::Excluded as u8;
        } else if c == 'g' {
            score.marks[i] = Mark::Green as u8;
        } else if c == 'y' {
            score.marks[i] = Mark::Yellow as u8;
        } else if c == '-' {
            score.marks[i] = Mark::NoScore as u8;
        } else {
            assert!(0 == 1);
        }
    }
    score
}

fn run_bot(words: &[Word], args: &[String]) {
    let mut nw: Vec<Word> = words.to_vec();
    let mut i: usize = 0;
    while i + 1 < args.len() {
        println!("{}: {}", args[i], args[i + 1]);
        let guess = encode_word(&args[i]);
        let score = cli_score(&guess, &args[i + 1]);
        nw = next_words(&nw, &score, &guess);
        let suggested = rankguesses(&nw, &nw)[0].0;
        println!{"-> bot move: {}", decode_word(&suggested)};
        i += 2;
    }
}

fn run_solve(words: &[Word], args: &[String]) {
    let word = args[0].as_str();
    let guess = args[1].as_str();
    println!("solve mode: word={}, guess={}", word, guess);
    solve(&words, &encode_word(word), &encode_word(guess));
}

fn run_calculate(words: &[Word], _args: &[String]) {
    let candidates = determine_candidates(&words);
    eprintln!("candidates: {} of {} total words", candidates.len(), words.len());
    let ranked = rankguesses(&candidates, &words);
    eprintln!("best word is: {}", decode_word(&ranked[0].0));
    println!("word,average_ratio");
    for (w, r) in ranked.iter() {
        println!("{},{}", decode_word(w), r);
    }
}

fn test() {
    let word = encode_word("crank");
    let guess = encode_word("raise");
    let candidate = encode_word("rayon");

    let ex = CharCount { count: [0; 26] };
    let score = mark_guess(&word, &guess);

    println!("{:?}", score);
    println!("{:?}", word_valid(&guess, &candidate, &ex, &score));
    return;
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} <wordlist> [bot|calculate|solve]", args[0]);
        return;
    }

    let words = read_words(args[1].as_str());
    let command = &args[2];
    if command == "bot" {
        run_bot(&words, &args[3..]);
    } else if command == "calculate" {
        run_calculate(&words, &args[3..]);
    } else if command == "solve" {
        run_solve(&words, &args[3..]);
    } else {
        println!("Unknown command: {}", command);
    }
}
