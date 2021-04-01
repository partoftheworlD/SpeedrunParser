use chrono::Utc;
use regex::Regex;
use std::{fs::File, io::Write, path::Path, thread};

fn filter(data: &str) -> Vec<String> {
    let re = Regex::new(r"game-name.*?<").unwrap();
    re.find_iter(data)
        .map(|x| -> String {
            x.as_str()
                .to_string()
                .replace("game-name\">", "")
                .replace("<", "\n")
        })
        .collect()
}

fn get_total() -> u32 {
    let re = Regex::new(r"(\d{0,},\d{3})").unwrap();
    let html = reqwest::blocking::get(
        "https://www.speedrun.com/ajax_games.php?
                            &platform=PC
                            &unofficial=off
                            &orderby=mostactive
                            &start=0",
    )
    .unwrap()
    .text()
    .unwrap();

    let total = re
        .find(html.as_str())
        .unwrap()
        .as_str()
        .replace(",", "")
        .parse::<u32>()
        .unwrap();
    println!("Amount of games on speedrun.com: {}", total);
    total
}

fn write_to_file(output_path: String, buffer: Vec<Vec<String>>) {
    let path = Path::new(&output_path);
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(file) => file,
    };
    buffer.into_iter().for_each(|i| {
        i.into_iter().for_each(|j| {
            match file.write_all(j.as_bytes()) {
                Err(why) => panic!("couldn't write to {}: {}", display, why),
                Ok(_) => (),
            };
        });
    });
}

fn run() {
    let n_workers = 4;
    let total_pages: f32 = get_total() as f32 / 50.0;
    let req_per_thread = (total_pages / n_workers as f32).round();
    let now = Utc::now().timestamp_millis();
    let mut pool = Vec::new();
    let mut offset = 0;

    for thread_id in 0..n_workers {
        let thread_handle = thread::spawn(move || {
            offset = thread_id * 50 * req_per_thread as u32;
            write_to_file(
                format!("speedruncom_pc_{}_part_{}.txt", now, thread_id + 1).to_string(),
                fun_name(offset, thread_id, req_per_thread as u32),
            );
        });
        pool.push(thread_handle);
    }
    for thread in pool {
        thread.join().unwrap();
    }
}

fn fun_name(offset: u32, thread_id: u32, req_per_thread: u32) -> Vec<Vec<String>> {
    let mut game_array = Vec::new();
    (offset..(thread_id + 1) * 50 * req_per_thread)
        .step_by(50)
        .for_each(|i| {
            let url = format!(
                "https://www.speedrun.com/ajax_games.php?platform=PC&unofficial=off&start={}",
                i
            );
            game_array.push(filter(
                &reqwest::blocking::get(&url)
                    .unwrap()
                    .text()
                    .unwrap()
                    .as_str(),
            ));
        });
    game_array
}

fn main() {
    run();
}
