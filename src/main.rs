extern crate egg_mode;
#[macro_use(object)] extern crate json;
use egg_mode::tweet::DraftTweet;
use egg_mode::Token;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::string::String;
use std::usize;
use json::JsonValue;
use std::borrow::Borrow;

// a beautiful mersenne prime
const PRIME: usize = 2147483647;

struct State<'a> {
  consumer_token: Option<Token<'a>>,
  access_token:   Option<Token<'a>>,
  counter:        usize,
}

fn serialize_token(t: &Token) -> JsonValue {
  object!{
      "key" => t.key.as_ref().borrow(),
      "secret" => t.secret.as_ref().borrow()
    }
}

fn serialize_state(s: &State) -> JsonValue {
  object!{
    "consumer_token" => JsonValue::from(s.consumer_token.as_ref().map(|t| serialize_token(&t))),
    "access_token" => JsonValue::from(s.access_token.as_ref().map(|t| serialize_token(&t))),
    "counter" => s.counter
  }
}

fn json_value_to_token(j: &JsonValue) -> Option<Token<'static>> {
  let key = j["key"].as_str().map(|s| s.to_string());
  let secret = j["secret"].as_str().map(|s| s.to_string());

  if key.is_none() || secret.is_none() {
    return None;
  }

  Some(Token::new(key.unwrap(), secret.unwrap()))
}

fn deserialize_state(s: &str) -> Option<State<'static>> {
  let parsed = match json::parse(s) {
    Ok(p) => p,
    Err(_) => return None
  };

  let state = State {
    consumer_token: json_value_to_token(&parsed["consumer_token"]),
    access_token:   json_value_to_token(&parsed["access_token"]),
    counter:        match parsed["counter"].as_number() {
      None => return None,
      Some(c) => c.into()
    }
  };

  Some(state)
}

fn read_state(p: &Path) -> Option<State<'static>> {
  let mut f = match File::open(p) {
    Ok(f) => f,
    Err(_) => return None
  };
  let mut s = String::new();

  match f.read_to_string(&mut s) {
    Err(_) => None,
    Ok(_) => deserialize_state(s.as_str())
  }

}

fn write_state(p: &Path, s: &State) {
  let mut f = match File::create(p) {
    Ok(f) => f,
    Err(e) => panic!("Could not create file: {}", e.to_string())
  };

  let res = f.write_fmt(format_args!("{}", serialize_state(s)));

  if res.is_err() {
    panic!("Could not write file: {}", res.err().unwrap().to_string());
  }
}

fn file_openable(p: &Path) -> bool {
  let file = File::open(p);

  file.is_ok()
}

fn file_lines(p: &Path) -> Option<usize> {
  match File::open(p) {
    Err(_) => None,
    Ok(file) => {
      let mut f = file;

      let mut s = String::new();
      match f.read_to_string(&mut s) {
        Err(_) => None,
        Ok(_) => Some(s.lines().count())
      }
    }
  }
}

fn next_line(counter: usize, lines: usize) -> usize {
  let limit = lines - 1;
  assert!(limit > 0);
  assert!(limit % PRIME != 0);

  (counter * PRIME) % limit
}

fn read_word(p: &Path, next: usize) -> Option<String> {
  match File::open(p) {
    Err(_) => None,
    Ok(file) => {
      let mut f = file;
      let mut s = String::new();
      match f.read_to_string(&mut s) {
        Err(_) => None,
        Ok(_) => s.lines().nth(next).map(|s| String::from(s))
      }
    }
  }
}

fn get_access_token(con: &Token) -> Option<Token<'static>> {
  let request_token = match egg_mode::request_token(con, "oob") {
    Err(_) => return None,
    Ok(r) => r
  };

  println!("Visit {} to authorize this bot! Enter the PIN below!", egg_mode::authorize_url(&request_token));

  let mut pin = String::new();

  match std::io::stdin().read_line(&mut pin) {
    Err(_) => None,
    Ok(_) => egg_mode::access_token(con, &request_token, pin).ok().map(|t| t.0)
  }
}

fn main() {
  let state_file     = Path::new("./state.json");
  let word_list_file = Path::new("./nouns");

  if !file_openable(word_list_file) {
    panic!("Need word list file at {}", word_list_file.to_str().unwrap());
  }

  if !file_openable(state_file) {
    write_state(&state_file, &State {
      consumer_token : None,
      access_token : None,
      counter : 0
    });
    println!("Please enter your application's consumer key and secret into {}", state_file.to_str().unwrap());
    return;
  }

  let word_count = file_lines(word_list_file).unwrap();
  let mut state  = read_state(&state_file).unwrap();

  if state.consumer_token.is_none() {
    panic!("No consumer token in {}!", state_file.to_str().unwrap());
  }

  if state.access_token.is_none() {
    state.access_token = get_access_token(state.consumer_token.as_ref().unwrap());

    if state.access_token.is_none() {
      panic!("Could not get access token!");
    }

  }

  let next_line = next_line(state.counter, word_count);
  let next_word = read_word(word_list_file, next_line).unwrap();
  let mut next_tweet = String::new();

  next_tweet.push_str(next_word.as_str());
  next_tweet.push_str(" oriented programming");

  assert!(state.counter + 1 <= word_count);

  let draft = DraftTweet::new(next_tweet.as_str());
  draft.send(state.consumer_token.as_ref().unwrap(), state.access_token.as_ref().unwrap()).unwrap();

  println!("Count of nouns: {}
Counter:         {}
Used Line:       {}
Tweet text:      {}", word_count, state.counter, next_line, draft.text);

  state.counter += 1;

  write_state(state_file, &state);
}
