use reqwest::blocking as reqwest;
use ::reqwest::header;
use scraper::{Html, Selector};
use regex::Regex;
use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::process::Command;
use std::env;
use indicatif::ProgressBar;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://chromium.woolyss.com/";
    let link = find_download_link(url)?;
    let filename = link.split('/').last().unwrap();

    if user_confirms_download(filename)? {
        download_file(&link, filename)?;
        execute_file(filename)?;
        ask_to_delete_file(filename)?;
    } else {
        println!("Download canceled.");
    }

    Ok(())
}

fn find_download_link(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get(url)?;
    let body = response.text()?;
    let document = Html::parse_document(&body);
    let a_selector = Selector::parse("a").unwrap();
    let re = Regex::new(r"_ungoogled_mini_installer\.exe")?;

    for element in document.select(&a_selector) {
        if let Some(title) = element.value().attr("title") {
            if re.is_match(title) {
                return Ok(element.value().attr("href").unwrap().to_string());
            }
        }
    }

    Err("Download link not found".into())
}

fn user_confirms_download(filename: &str) -> Result<bool, io::Error> {
    println!("File Name: {}", filename);
    print!("Do you want to download this file? (y/n): ");
    io::stdout().flush()?;
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    Ok(choice.trim().to_lowercase().starts_with('y'))
}

fn download_file(link: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let head_response = reqwest::get(link)?;
    let total_size = head_response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| val.parse().ok())
        .unwrap_or(0);

    let mut file_response = reqwest::get(link)?;
    let mut file = File::create(filename)?;
    let progress_bar = ProgressBar::new(total_size as u64);

    let mut buffer = vec![0; 8192];
    while let Ok(bytes_read) = file_response.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        progress_bar.inc(bytes_read as u64);
    }

    progress_bar.finish();
    println!("\nDownloaded: {}", filename);
    
    // Explicitly close the file
    drop(file);
    
    Ok(())
}

fn execute_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let full_path = current_dir.join(filename);

    // Add a small delay before executing
    thread::sleep(Duration::from_secs(1));

    // Use 'open' command on Windows to execute the file
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", "start", "", full_path.to_str().unwrap()])
            .status()?
    } else {
        Command::new(full_path).status()?
    };

    if status.success() {
        println!("Successfully executed file: {}", filename);
    } else {
        println!("Error while executing file: {}", status);
    }

    Ok(())
}

fn ask_to_delete_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    print!("Do you want to delete the file '{}'? (y/n): ", filename);
    io::stdout().flush()?;
    let mut delete_choice = String::new();
    io::stdin().read_line(&mut delete_choice)?;
    
    if delete_choice.trim().to_lowercase().starts_with('y') {
        fs::remove_file(filename)?;
        println!("File '{}' has been deleted.", filename);
    }

    Ok(())
}