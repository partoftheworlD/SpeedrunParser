use chrono::Utc;
use regex::Regex;
use std::{fs::File, io::Write, path::Path, thread};

const PAGE_SIZE: u32 = 50;

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

fn write_to_file(output_path: String, g_box: Box<Vec<Vec<String>>>) {
    let path = Path::new(&output_path);
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(file) => file,
    };

    g_box.into_iter().for_each(|unbox| {
        unbox.into_iter().for_each(|string| {
            match file.write_all(string.as_bytes()) {
                Err(why) => panic!("couldn't write to {}: {}", display, why),
                Ok(_) => (),
            };
        })
    });
}

fn run() {
    const WORKERS: u32 = 8;
    let now = Utc::now().timestamp_millis();
    let total_pages = (get_total() as f32 / PAGE_SIZE as f32).round();
    let req_per_thread = (total_pages / WORKERS as f32).round() as u32;

    let thread_handles: Vec<_> = (0..WORKERS)
        .map(|thread_id| {
            thread::spawn(move || {
                let offset = thread_id * PAGE_SIZE * req_per_thread;
                write_to_file(
                    format!("speedruncom_pc_{}_part_{}.txt", now, thread_id + 1).to_string(),
                    make_request(offset, thread_id, req_per_thread),
                );
            })
        })
        .collect();

    for thread in thread_handles {
        thread.join().unwrap();
    }
}

fn make_request(offset: u32, thread_id: u32, req_per_thread: u32) -> Box<Vec<Vec<String>>> {
    let mut g_box = Box::new(Vec::new());

    (offset..(thread_id + 1) * PAGE_SIZE * req_per_thread)
        .step_by(PAGE_SIZE as usize)
        .for_each(|page| {
            let url = format!(
                "https://www.speedrun.com/ajax_games.php?platform=PC&unofficial=off&start={}",
                page
            );
            g_box.push(filter(
                &reqwest::blocking::get(&url)
                    .unwrap()
                    .text()
                    .unwrap()
                    .as_str(),
            ));
        });
    g_box
}

fn main() {
    run();
}
