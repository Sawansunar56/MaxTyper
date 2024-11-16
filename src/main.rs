use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Duration;
use rand::{thread_rng};
use rand::seq::IndexedRandom;
use crossterm::{ cursor, terminal, event, execute, queue, style::{self, Color, Print } };
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::style::SetForegroundColor;

fn read_file_for_words<P>(filename: P) -> io::Result<Vec<String>>
where P: AsRef<Path>,
{
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let words: Vec<String> = contents.split_whitespace().map(|s| s.to_string()).collect();
    Ok(words)
}

fn main() -> io::Result<()> {
    let filename = "words.txt";
    let words = read_file_for_words(filename).expect("Something went wrong reading the file");

    if words.is_empty() {
        println!("No words found.");
        return Ok(());
    }

    let mut rng = thread_rng();
    // let random_word = words.choose(&mut rng).expect("Array is empty");
    let random_word = "Something to do with life";
    println!("{}", random_word);

    let mut stdout = io::stdout();

    execute!(stdout, terminal::Clear(terminal::ClearType::All));

    for word in random_word.split_whitespace() {
        queue!(stdout, style::SetForegroundColor(Color::Grey), Print(word), Print(" "))?;
    }
    stdout.flush()?;
    let mut is_word_correct = true;

    let mut current_position = 0;
    let mut typed_word = String::new();
    loop {
        if event::poll(Duration::from_millis(300))? {
            if let Event::Key(KeyEvent { code, kind, .. }) = event::read()? {
                if kind != event::KeyEventKind::Press {
                    continue;
                }
                match code {
                    KeyCode::Esc => break,
                    KeyCode::Char(c) => {
                        if current_position < random_word.len() {
                            let target_char = random_word.chars().nth(current_position).unwrap();

                            execute!(stdout, cursor::MoveTo(current_position as u16, 0))?;

                            let is_correct = c == target_char;
                            if !is_correct {
                                is_word_correct = false;
                            }

                            let color = if is_correct {
                                Color::White
                            } else {
                                Color::Red
                            };

                            execute!(stdout, SetForegroundColor(color))?;
                            if target_char == ' ' && !is_word_correct {
                                print!("_");
                            } else {
                                print!("{}", target_char);
                            }

                            typed_word.push(target_char);
                            current_position += 1;

                            execute!(stdout, cursor::MoveTo(current_position as u16, 0))?;
                        }
                    }
                    KeyCode::Backspace => {
                        if current_position > 0 {
                            current_position -= 1;
                            typed_word.pop();

                            is_word_correct = typed_word.chars().zip(random_word.chars()).all(|(c1, c2)| c1 == c2);

                            let target_chars: Vec<char> = random_word.chars().collect();
                            let target_char = target_chars[current_position];

                            execute!(stdout, cursor::MoveTo(current_position as u16, 0), SetForegroundColor(Color::Grey))?;
                            print!("{}", target_char);

                            execute!(stdout, cursor::MoveTo(current_position as u16, 0))?;
                        }
                    }
                    _ => {}
                }
            }
        }

        if current_position == random_word.len() {
            execute!(stdout, cursor::MoveTo(0, 2))?;
            // This is a wrong comparison
            if is_word_correct {
                println!("You have completed");
                println!("{}", typed_word);
            } else {
                print!("You are wrong");
            }
            break;
        }
    }
    Ok(())
}
