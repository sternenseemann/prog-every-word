extern crate egg_mode;
use egg_mode::tweet::DraftTweet;
use egg_mode::Token;
use std::option;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::string::String;
use std::str::FromStr;
use std::usize;

// a beautiful mersenne prime
const prime: usize = 2147483647;

fn read_token(p: &Path) -> Option<Token<'static>> {
    let file = File::open(p);

    if file.is_err() {
      return None;
    }

    let mut f = file.unwrap();

    let mut s = String::new();
    let res = f.read_to_string(&mut s);

    if res.is_err() {
      return None;
    }

    let mut lines = s.lines();
    let key = lines.next();
    let secret = lines.next();

    if key.is_none() || secret.is_none() {
      return None;
    }

    Some(Token::new(key.unwrap().to_string(), secret.unwrap().to_string()))
}

fn read_counter(p: &Path) -> Option<usize> {
  let file = File::open(p);

  if file.is_err() {
    return None;
  }

  let mut f = file.unwrap();

  let mut s = String::new();
  let res = f.read_to_string(&mut s);

  if res.is_err() {
    return None;
  }

  let number = usize::from_str(s.as_str());

  number.ok()
}

fn write_counter(p: &Path, counter: usize) {
  let mut file = File::create(p).unwrap();

  file.write_fmt(format_args!("{}", counter));
}

fn file_lines(p: &Path) -> Option<usize> {
  let file = File::open(p);

  if file.is_err() {
    return None;
  }

  let mut f = file.unwrap();

  let mut s = String::new();
  let res = f.read_to_string(&mut s);

  if res.is_err() {
    return None;
  }

  Some(s.lines().count())
}

fn next_line(counter: usize, lines: usize) -> usize {
  let limit = lines - 1;
  assert!(limit > 0);
  assert!(limit % prime != 0);

  (counter * prime) % limit
}

fn read_word(p: &Path, next: usize) -> Option<String> {
  let file = File::open(p);

  if file.is_err() {
    return None;
  }

  let mut f = file.unwrap();
  let mut s = String::new();
  let res = f.read_to_string(&mut s);

  if res.is_err() {
    return None;
  }

  s.lines().nth(next).map(|s| String::from(s))
}

fn get_access_token(con: &Token) -> Option<Token<'static>> {
  let request_token = match egg_mode::request_token(con, "oob") {
    Err(e) => return None,
    Ok(r) => r
  };

  println!("Visit {} to authorize this bot! Enter the PIN below!", egg_mode::authorize_url(&request_token));

  let mut pin = String::new();

  if std::io::stdin().read_line(&mut pin).is_err() {
    return None;
  }

  egg_mode::access_token(con, &request_token, pin).ok().map(|t| t.0)

}

fn write_token(p: &Path, t: &Token) {
  let mut file = File::create(p);
  if file.is_err() {
    return;
  }

  let mut f = file.unwrap();

  f.write_fmt(format_args!("{}\n{}", t.key, t.secret));

}

fn main() {
  let consumer_token_file = Path::new("./state/consumer_token");
  let access_token_file   = Path::new("./state/access_token");
  let state_file          = Path::new("./state/counter");
  let word_list_file      = Path::new("./state/nouns");

  let consumer_token = read_token(consumer_token_file).unwrap();
  let access_token   = match read_token(access_token_file) {
    Some(t) => t,
    None => {
      let acc_token = get_access_token(&consumer_token).unwrap();

      write_token(access_token_file, &acc_token);

      acc_token
    }
  };

  let counter        = match read_counter(state_file) {
    Some(i) => i,
    None => {
      let f = File::create(state_file);
      if f.is_err() {
        panic!("Could not create state file \"{}\"", state_file.to_str().unwrap());
      }
      f.unwrap().write_fmt(format_args!("0"));
      read_counter(state_file).unwrap()
    }
  };
  let word_count = file_lines(word_list_file).unwrap();

  let next_line = next_line(counter, word_count);
  let next_word = read_word(word_list_file, next_line).unwrap();
  let mut next_tweet = String::new();

  next_tweet.push_str(next_word.as_str());
  next_tweet.push_str(" oriented programming");

  assert!(counter + 1 <= word_count);

  let draft = DraftTweet::new(next_tweet.as_str());
  draft.send(&consumer_token, &access_token).unwrap();

  println!("Count of nouns: {}
Counter:         {}
Used Line:       {}
Tweet text:      {}", word_count, counter, next_line, draft.text);

  write_counter(state_file, counter + 1);
}
