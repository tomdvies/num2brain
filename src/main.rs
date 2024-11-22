use colored::*;
use ctrlc;
use rand::Rng;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static TOTAL_QUESTIONS: AtomicUsize = AtomicUsize::new(0);
static CORRECT_ANSWERS: AtomicUsize = AtomicUsize::new(0);
static START_TIME: AtomicUsize = AtomicUsize::new(0); // Stores duration since UNIX epoch in seconds
                                                      // Number ranges for different operations
const MULTIPLICATION_MIN: i32 = 2;
const MULTIPLICATION_MAX: i32 = 13;
const ADDITION_MIN: i32 = 1;
const ADDITION_MAX: i32 = 100;
const DIVISION_MAX_MULTIPLIER: i32 = 13;
const DIVISION_MAX_DIVISOR: i32 = 13;

struct PracticeConfig {
    num_questions: Option<i32>,
    time_limit: Option<Duration>,
}

impl Default for PracticeConfig {
    fn default() -> Self {
        PracticeConfig {
            num_questions: None,
            time_limit: Some(Duration::from_secs(600)), // 10 minutes
        }
    }
}
fn get_practice_config() -> PracticeConfig {
    println!(
        "\n{}",
        "=== Practice Configuration ===".bright_cyan().bold()
    );
    println!("(At least one of these must be set)");

    print!("Enter number of questions (press Enter for unlimited): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let num_questions = if input.trim().is_empty() {
        None
    } else {
        Some(input.trim().parse().unwrap_or(5))
    };

    print!("Enter time limit in minutes (press Enter for no limit): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let time_limit = if input.trim().is_empty() {
        None
    } else {
        match input.trim().parse::<u64>() {
            Ok(minutes) => Some(Duration::from_secs(minutes * 60)),
            Err(_) => None,
        }
    };

    if num_questions.is_none() && time_limit.is_none() {
        println!(
            "{}",
            "Setting default 10 minutes."
                .bright_magenta()
                .bold()
        );
        PracticeConfig {
            num_questions: None,
            time_limit: Some(Duration::from_secs(600)), // 10 minutes
        }
    } else {
        PracticeConfig {
            num_questions,
            time_limit,
        }
    }
}

fn setup_signal_handler() {
    ctrlc::set_handler(move || {
        let total = TOTAL_QUESTIONS.load(Ordering::Relaxed);
        let correct = CORRECT_ANSWERS.load(Ordering::Relaxed);
        let start_time = START_TIME.load(Ordering::Relaxed);

        if total > 0 {
            let elapsed = Duration::from_secs(
                Instant::now()
                    .elapsed()
                    .as_secs()
                    .saturating_sub(start_time as u64),
            );
            println!("\n\n{}", "=== Session Summary ===".bright_cyan().bold());
            println!("Total questions: {}", total);
            println!("Correct answers: {}", correct);
            println!("Accuracy: {:.1}%", (correct as f64 / total as f64) * 100.0);
            println!("Total time: {:.1} seconds", elapsed.as_secs_f32());
        }

        println!("\nGoodbye!");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");
}

fn main() {
    setup_signal_handler();
    loop {
        println!("\n{}", "=== Math Practice App ===".bright_green().bold());
        println!("1. Multiplication Tables");
        println!("2. Addition Practice");
        println!("3. Division Practice");
        println!("4. Mixed Practice (All Operations)");
        println!("5. Kelly Bet Practice");
        println!("6. Quit");
        print!("\nChoose an option (1-6): ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => practice_multiplication(),
            "2" => practice_addition(),
            "3" => practice_division(),
            "4" => practice_mixed(),
            "5" => practice_kelly_bet(),
            "6" => {
                println!("{}", "Goodbye!".bright_yellow().bold());
                break;
            }
            _ => println!("Invalid choice! Please try again."),
        }
    }
}

fn get_user_answer() -> i32 {
    loop {
        print!("Your answer: ");
        io::stdout().flush().unwrap();
        let mut answer = String::new();
        io::stdin().read_line(&mut answer).unwrap();

        match answer.trim().parse() {
            Ok(num) => return num,
            Err(_) => println!("Please enter a valid number!"),
        }
    }
}

struct Question {
    num1: i32,
    num2: i32,
    operator: &'static str,
    correct_answer: i32,
}

fn generate_multiplication_question() -> Question {
    let mut rng = rand::thread_rng();
    let num1 = rng.gen_range(MULTIPLICATION_MIN..MULTIPLICATION_MAX);
    let num2 = rng.gen_range(MULTIPLICATION_MIN..MULTIPLICATION_MAX);
    Question {
        num1,
        num2,
        operator: "×",
        correct_answer: num1 * num2,
    }
}

fn generate_addition_question() -> Question {
    let mut rng = rand::thread_rng();
    let num1 = rng.gen_range(ADDITION_MIN..ADDITION_MAX);
    let num2 = rng.gen_range(ADDITION_MIN..ADDITION_MAX);
    Question {
        num1,
        num2,
        operator: "+",
        correct_answer: num1 + num2,
    }
}

fn generate_subtraction_question() -> Question {
    let mut rng = rand::thread_rng();
    let num1 = rng.gen_range(ADDITION_MIN..ADDITION_MAX);
    let num2 = rng.gen_range(1..num1);
    Question {
        num1,
        num2,
        operator: "-",
        correct_answer: num1 - num2,
    }
}

fn generate_division_question() -> Question {
    let mut rng = rand::thread_rng();
    let num2 = rng.gen_range(1..DIVISION_MAX_DIVISOR);
    let num1 = num2 * rng.gen_range(1..DIVISION_MAX_MULTIPLIER);
    Question {
        num1,
        num2,
        operator: "÷",
        correct_answer: num1 / num2,
    }
}

fn practice_questions(practice_type: &str, generator: fn() -> Question, config: PracticeConfig) {
    let mut correct = 0;
    let start_time = Instant::now();
    START_TIME.store(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
        Ordering::Relaxed,
    );

    println!(
        "\n{}",
        format!("=== {} Practice ===", practice_type)
            .bright_blue()
            .bold()
    );
    if let Some(limit) = config.time_limit {
        println!("Time limit: {} minutes", limit.as_secs() / 60);
    }

    let mut i = 1;
    while config.num_questions.map_or(true, |n| i <= n) {
        if let Some(limit) = config.time_limit {
            if start_time.elapsed() >= limit {
                println!("\n{}", "Time's up!".yellow().bold());
                break;
            }
        }
        TOTAL_QUESTIONS.fetch_add(1, Ordering::Relaxed);

        let question = generator();
        println!(
            "\n{} What is {} {} {}?",
            format!("Question {}:", i).bright_green().bold(),
            question.num1,
            question.operator,
            question.num2
        );

        let user_answer = get_user_answer();

        if user_answer == question.correct_answer {
            println!("{}", "Correct! ✓".bright_green().bold());
            correct += 1;
            CORRECT_ANSWERS.fetch_add(1, Ordering::Relaxed);
        } else {
            println!(
                "{} {}",
                "Wrong! The correct answer is".bright_red().bold(),
                question.correct_answer
            );
        }
        i += 1;
    }

    let elapsed = start_time.elapsed();
    let score_message = match config.num_questions {
        Some(n) => format!("You got {}/{} correct!", correct, n),
        None => format!("You got {} correct!", correct),
    };
    println!("\n{}", score_message.bright_purple().bold());
    println!("Time taken: {:.1} seconds", elapsed.as_secs_f32());
}

fn practice_multiplication() {
    let config = get_practice_config();
    practice_questions("Multiplication", generate_multiplication_question, config);
}

fn practice_addition() {
    let config = get_practice_config();
    practice_questions("Addition", generate_addition_question, config);
}

fn practice_mixed() {
    let config = get_practice_config();
    let mut rng = rand::thread_rng();
    let mut correct = 0;
    let start_time = Instant::now();

    println!("\n=== Mixed Practice (All Operations) ===");
    if let Some(limit) = config.time_limit {
        println!("Time limit: {} minutes", limit.as_secs() / 60);
    }

    let mut i = 1;
    while config.num_questions.map_or(true, |n| i <= n) {
        if let Some(limit) = config.time_limit {
            if start_time.elapsed() >= limit {
                println!("\n{}", "Time's up!".yellow().bold());
                break;
            }
        }
        let operation = rng.gen_range(1..=4);

        let question = match operation {
            1 => generate_multiplication_question(),
            2 => generate_addition_question(),
            3 => generate_subtraction_question(),
            4 => generate_division_question(),
            _ => unreachable!(),
        };

        println!(
            "\n{} What is {} {} {}?",
            format!("Question {}:", i).bright_green().bold(),
            question.num1,
            question.operator,
            question.num2
        );
        let user_answer = get_user_answer();

        if user_answer == question.correct_answer {
            println!("{}", "Correct! ✓".bright_green().bold());
            correct += 1;
        } else {
            println!(
                "{} {}",
                "Wrong! The correct answer is".bright_red().bold(),
                question.correct_answer
            );
        }
        i += 1;
    }
    let score_message = match config.num_questions {
        Some(n) => format!("You got {}/{} correct!", correct, n),
        None => format!("You got {} correct!", correct),
    };
    println!("\n{}", score_message.bright_purple().bold());
}

fn practice_division() {
    let config = get_practice_config();
    practice_questions("Division", generate_division_question, config);
}

fn practice_kelly_bet() {
    let config = get_practice_config();
    let mut rng = rand::thread_rng();
    let mut correct = 0;
    let start_time = Instant::now();
    let tolerance = 0.05; // Accept answers within ±5%

    println!("\n{}", "=== Kelly Bet Practice ===".bright_blue().bold());
    println!("Try to calculate the optimal Kelly bet fraction.");
    println!("(Enter your answer as a decimal, e.g., 0.25 for 25%)");

    if let Some(limit) = config.time_limit {
        println!("Time limit: {} minutes", limit.as_secs() / 60);
    }

    let mut i = 1;
    while config.num_questions.map_or(true, |n| i <= n) {
        if let Some(limit) = config.time_limit {
            if start_time.elapsed() >= limit {
                println!("\n{}", "Time's up!".yellow().bold());
                break;
            }
        }

        // Generate random odds (between 1.5 and 5.0) and probability (between 0.4 and 0.8)
        let decimal_odds = rng.gen_range(1.5..5.0);
        let win_prob = rng.gen_range(0.2..0.8);

        // Calculate true Kelly fraction: (bp - q)/b where:
        // b = decimal odds - 1
        // p = probability of winning
        // q = probability of losing (1-p)
        let b = decimal_odds - 1.0;
        let p = win_prob;
        let q = 1.0 - p;
        let kelly_fraction = f64::max((b * p - q) / b, 0.0);

        println!("\n{}", format!("Question {}:", i).bright_green().bold());
        println!("Decimal Odds: {:.2}", decimal_odds);
        println!("Win Probability: {:.2}", win_prob);

        loop {
            print!("Your answer (as decimal): ");
            io::stdout().flush().unwrap();
            let mut answer = String::new();
            io::stdin().read_line(&mut answer).unwrap();

            match answer.trim().parse::<f64>() {
                Ok(user_answer) => {
                    let diff = (user_answer - kelly_fraction).abs();
                    if diff <= tolerance {
                        println!("{}", "Correct! ✓".bright_green().bold());
                        correct += 1;
                    } else {
                        // color based on how far off the answer is
                        // > 20% is full red
                        let max_error = 0.20;
                        let accuracy = 1.0 - (diff / max_error).min(1.0);

                        let response = format!("{:.3}", kelly_fraction);
                        let colored_response = if accuracy < 0.33 {
                            response.bright_red()
                        } else if accuracy < 0.66 {
                            response.bright_yellow()
                        } else {
                            response.bright_green()
                        };

                        println!(
                            "{} {}",
                            "Wrong! The correct Kelly fraction is".bright_red().bold(),
                            colored_response.bold()
                        );
                    }
                    break;
                }
                Err(_) => println!("Please enter a valid decimal number!"),
            }
        }
        i += 1;
    }

    let score_message = match config.num_questions {
        Some(n) => format!("You got {}/{} correct!", correct, n),
        None => format!("You got {} correct!", correct),
    };
    println!("\n{}", score_message.bright_purple().bold());
    println!(
        "Time taken: {:.1} seconds",
        start_time.elapsed().as_secs_f32()
    );
}
