use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::style::SetForegroundColor;
use crossterm::{
    cursor, event, execute, queue,
    style::{self, Color, Print},
    terminal,
};
use rand::seq::IndexedRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::fs::File;
use std::io::{self, read_to_string, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};
// use std::thread::sleep;

#[derive(Serialize, Deserialize, Eq, PartialEq)]
struct WordData {
    key: String,
    value: i32,
}

impl WordData {
    fn new(key: String, value: i32) -> WordData {
        WordData { key, value }
    }
}

impl Ord for WordData {
    fn cmp(&self, other: &WordData) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for WordData {
    fn partial_cmp(&self, other: &WordData) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Serialize, Deserialize)]
struct MainData {
    entries: Vec<WordData>,
}

impl MainData {
    fn new() -> Self {
        MainData {
            entries: Vec::new(),
        }
    }

    fn add(&mut self, key: String, value: i32) {
        self.entries.push(WordData::new(key, value));
    }

    fn sort_by_value(&mut self) {
        self.entries.sort_by(|a, b| a.value.cmp(&b.value));
    }

    fn export_data(&self, filepath: &str) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(filepath)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn import_data(filepath: &str) -> io::Result<Self> {
        let file_content = fs::read_to_string(filepath)?;
        let database: MainData = serde_json::from_str(&file_content)?;
        Ok(database)
    }
}

fn read_file_for_words<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let words: Vec<String> = contents.split_whitespace().map(|s| s.to_string()).collect();
    Ok(words)
}

enum _RaceType {
    RaceTime,
    RaceWords,
}

enum SenGenType {
    Mistake,
    Random,
}

enum GameStage {
    Menu,
    Game,
}

fn main() -> io::Result<()> {
    let cache_file_name = "cache.json";
    let mut main_data = MainData::new();
    let path = Path::new(cache_file_name);
    let main_color = Color::White;
    let alternate_color = Color::DarkGrey;
    let error_color = Color::Red;

    // loading the data into memory
    if !path.exists() {
        let filename = "words.txt";
        let words = read_file_for_words(filename).expect("Something went wrong reading the file");

        if words.is_empty() {
            println!("No words found.");
            return Ok(());
        }
        for word in words {
            main_data.add(word, 0);
        }
    } else {
        main_data =
            MainData::import_data(&cache_file_name).expect("Something went wrong reading the file");
    }

    // sentence for current test.
    let word_count: u8 = 20;
    let mut current_sentence = String::new();
    let mut current_word_index: u8 = 0;
    let _mistake_truth = true;
    let game_type = SenGenType::Mistake;

    // TODO: Make current_word_index flexible with the random word generation as
    // well as the normal wrong sorted values.
    // NOTE: This is a good structure I think. The sentence to be rendered can be
    // prepared in here and just all the rendering stuff for later. And the addition
    // and negative thingy here.

    // Sentence Generation Code here.
    match game_type {
        SenGenType::Random => {
            for _ in 0..word_count {
                if let Some(random_word) = main_data.entries.choose(&mut thread_rng()) {
                    current_sentence.push_str(&random_word.key);
                    current_sentence.push_str(" ");
                }
            }
        }
        SenGenType::Mistake => {
            // sown: just generates negative value in order.
            for item in main_data.entries.iter_mut().take(word_count as usize) {
                let word = item.key.as_str().to_owned() + " ";
                current_sentence.push_str(&word);
            }
        }
    }

    let mut stdout = io::stdout();

    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    let mut current_position = 0;
    let mut typed_word = String::new();

    // FIX: This damn random word is a wrong way to name this. Probably need to 
    // fix it when I am revamping the game implementation.
    let mut random_word = current_sentence;

    // for word in random_word.split_whitespace() {
    //     queue!(stdout, style::SetForegroundColor(Color::Grey), cursor::MoveTo(current_position as u16, 0), Print(word))?;
    // }
    queue!(
        stdout,
        style::SetForegroundColor(alternate_color),
        cursor::MoveTo(current_position as u16, 0),
        Print(random_word.clone())
    )?;
    stdout.flush()?;
    let mut is_word_correct = true;

    // Make this more generic so that I can make even more options like on timer
    // on para completion, on speed.
    let mut start_time = Instant::now();
    let mut timer_init: bool = false;

    let game_mode = GameStage::Menu;

    loop {
        // Okay I need to make a normal menu stage where I can choose everything
        // else and then I need to have a game stage.
        // TODO: Tab should restart. Esc should go to menu and then quit.
        match game_mode {
            GameStage::Menu => {
                let menu_name = String::from("Do you wanna start?");
                if let Event::Key(KeyEvent { code, kind, .. }) = event::read()? {
                    if kind != event::KeyEventKind::Press {
                        queue!(
                            stdout,
                            terminal::Clear(terminal::ClearType::All),
                            style::SetForegroundColor(main_color),
                            cursor::MoveTo(current_position as u16, 0),
                            Print(menu_name)
                        )?;
                        stdout.flush()?;
                        continue;
                    }
                }
                // if let Event::Key(KeyEvent { code, kind, .. }) = event::read()? {
                //     if kind !=
                // }
            }
            GameStage::Game => continue,
        }

        // if event::poll(Duration::from_millis(100))? {
        if let Event::Key(KeyEvent { code, kind, .. }) = event::read()? {
            if kind != event::KeyEventKind::Press {
                continue;
            }

            match code {
                KeyCode::Esc => break,
                KeyCode::Tab => {
                    // Restart system
                    // Maybe make reset function.
                    current_word_index = 0;
                    current_position = 0;
                    main_data.sort_by_value();
                    main_data.export_data(cache_file_name)?;
                    let mut new_sentence = String::new();
                    for item in main_data.entries.iter_mut().take(word_count as usize) {
                        let word = item.key.as_str().to_owned() + " ";
                        new_sentence.push_str(&word);
                    }

                    random_word = new_sentence;
                    queue!(
                        stdout,
                        style::SetForegroundColor(alternate_color),
                        terminal::Clear(terminal::ClearType::All),
                        cursor::MoveTo(current_position as u16, 0),
                        Print(&random_word),
                        cursor::MoveTo(current_position as u16, 0),
                    )?;
                    stdout.flush()?;
                    // print!("{}", random_word);
                    // execute!(stdout, cursor::MoveTo(current_position as u16, 0))?;
                    // stdout.flush()?;
                    // queue!(
                    //     stdout,
                    //     style::SetForegroundColor(Color::Grey),
                    //     cursor::MoveTo(current_position as u16, 0),
                    //     Print(random_word.clone())
                    // )?;
                    typed_word.clear();
                    start_time = Instant::now();
                }
                KeyCode::Char(c) => {
                    if !timer_init {
                        timer_init = true;
                        start_time = Instant::now();
                    }
                    if current_position < random_word.len() {
                        let target_char = random_word.chars().nth(current_position).unwrap();

                        execute!(stdout, cursor::MoveTo(current_position as u16, 0))?;

                        let is_correct = c == target_char;
                        let color = if is_correct { main_color } else { error_color };
                        execute!(stdout, SetForegroundColor(color))?;

                        if !is_correct {
                            is_word_correct = false;
                            main_data.entries[current_word_index as usize].value -= 2;
                        }

                        if target_char == ' ' {
                            if !is_correct {
                                print!("_");
                            } else {
                                if is_word_correct {
                                    main_data.entries[current_word_index as usize].value += 1;
                                }
                                current_word_index += 1;
                                print!("{}", target_char);
                            }
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

                        is_word_correct = typed_word
                            .chars()
                            .zip(random_word.chars())
                            .all(|(c1, c2)| c1 == c2);

                        let target_chars: Vec<char> = random_word.chars().collect();
                        let target_char = target_chars[current_position];

                        if target_char == ' ' {
                            current_word_index -= 1;
                        }

                        execute!(
                            stdout,
                            cursor::MoveTo(current_position as u16, 0),
                            SetForegroundColor(alternate_color)
                        )?;
                        print!("{}", target_char);

                        execute!(stdout, cursor::MoveTo(current_position as u16, 0))?;
                    }
                }
                _ => {}
            }
        }
        // }

        if current_position == random_word.len() {
            execute!(stdout, cursor::MoveTo(0, 2))?;
            // This is a wrong comparison
            if is_word_correct {
                println!("You have completed");
                // println!("{}", typed_word);
            } else {
                println!("You are wrong");
            }
            break;
        }
    }

    let end_time = Instant::now();
    let duration = end_time - start_time;
    println!("Duration: {}s", duration.as_secs());
    let wpm: f32 = word_count as f32 / (duration.as_secs_f32() / 60.0);
    println!("WPM: {}", wpm);

    if !main_data.entries.is_empty() {
        println!("Saved File");
        main_data.sort_by_value();
        main_data.export_data(cache_file_name)?;
    }

    queue!(stdout, style::SetForegroundColor(main_color))?;

    Ok(())
}
