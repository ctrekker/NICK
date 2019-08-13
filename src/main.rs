// NICK - New Integrated Context Karyover
//   The acronym sucks, and so does the workflow

use clap::{App, SubCommand, Arg, ArgMatches, AppSettings};
use rusqlite::{params, Connection};
use std::{fs, thread};
use std::path::Path;
use std::error::Error;
use std::net::{TcpListener, TcpStream, Shutdown, SocketAddr};
use std::io::{Read, Write, BufReader, BufWriter};
use std::sync::{Mutex, Arc, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::{File, Metadata};
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use byte::{BytesExt, BE};
use data_encoding::HEXUPPER;
use byte::ctx::{Str, NULL};
use ring::digest::{Context, Digest, SHA256};
use tar::Archive;
use std::thread::JoinHandle;

fn unpack() -> Result<(), Box<dyn Error>>  {
    let data = File::open("/Users/cburns/.nick/tmp/1562191683833.tar.gz")?;
    let decompressed = GzDecoder::new(data);
    let mut archive = Archive::new(decompressed);
    archive.unpack("/Users/cburns/Documents/Programming/Grapher")?;

    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
//    unpack()?;
//    return Ok(());

    let db: Connection = (init_database()?).unwrap();
    let matches = App::new("NICK")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("New Integrated Context Karyover")
        .subcommand(SubCommand::with_name("init")
            .about("Initialize project")
            .arg(Arg::with_name("path")
                .short("p")
                .long("path")
                .takes_value(true)
                .value_name("PATH")
                .help("Path of the project root. Defaults to PWD (present working directory)"))
            .arg(Arg::with_name("name")
                .short("n")
                .long("name")
                .takes_value(true)
                .value_name("NAME")
                .help("Name of project. Used for remote access to project")))
        .subcommand(SubCommand::with_name("server")
            .setting(AppSettings::ArgRequiredElseHelp)
            .about("Manage internal server")
            .subcommand(SubCommand::with_name("start")
                .about("Starts the internal server"))
            .subcommand(SubCommand::with_name("status")
                .about("Gets current status of the internal server")
                .arg(Arg::with_name("remote")
                    .help("Name of remote to check status of")
                    .value_name("REMOTE")
                    .index(1))))
        .subcommand(SubCommand::with_name("remotes")
            .setting(AppSettings::ArgRequiredElseHelp)
            .about("Manages the remote locations with which to sync")
            .subcommand(SubCommand::with_name("add")
                .about("Creates a new remote")
                .arg(Arg::with_name("name")
                    .help("Name of the remote. Used for identification only")
                    .value_name("NAME")
                    .required(true)
                    .index(1))
                .arg(Arg::with_name("url")
                    .help("URL of the remote to add (127.0.0.1")
                    .value_name("URL")
                    .required(true)
                    .index(2)))
            .subcommand(SubCommand::with_name("list")
                .about("Lists all remotes")
                .arg(Arg::with_name("simple")
                    .short("s")
                    .help("Simplifies output into an easily parsable form")))
            .subcommand(SubCommand::with_name("update")
                .about("Modifies a remote")
                .arg(Arg::with_name("name")
                    .help("Name of the remote to modify")
                    .value_name("NAME")
                    .required(true)
                    .index(1))
                .arg(Arg::with_name("new_name")
                    .short("n")
                    .long("name")
                    .takes_value(true)
                    .value_name("NEW_NAME")
                    .help("New name for the remote"))
                .arg(Arg::with_name("new_url")
                    .short("u")
                    .long("url")
                    .takes_value(true)
                    .value_name("NEW_URL")
                    .help("New url for the remote")))
            .subcommand(SubCommand::with_name("delete")
                .about("Deletes a remote")
                .arg(Arg::with_name("name")
                    .help("Name of the remote to delete")
                    .value_name("NAME")
                    .required(true)
                    .index(1))))
        .subcommand(SubCommand::with_name("sync")
            .setting(AppSettings::ArgRequiredElseHelp)
            .about("Syncs current code repo with remotes. Defaults to all remotes")
            .subcommand(SubCommand::with_name("up")
                .about("Syncs local code up to remote code. Overrides remote code")
                .arg(Arg::with_name("remote")
                    .help("Name of the remote to sync code to")
                    .value_name("REMOTE")
                    .required(true)
                    .index(1))
                .arg(Arg::with_name("project")
                    .help("Name for project on remote. Defaults to current project's name")
                    .short("p")
                    .long("project")
                    .takes_value(true)
                    .value_name("PROJECT")))
            .subcommand(SubCommand::with_name("down")
                .about("Syncs remote code down to local code. Overrides local code")
                .arg(Arg::with_name("remote")
                    .help("Name of the remote to sync code from")
                    .value_name("REMOTE")
                    .required(true)
                    .index(1))
                .arg(Arg::with_name("project")
                    .help("Name for project on remote. Defaults to current project's name")
                    .short("p")
                    .long("project")
                    .takes_value(true)
                    .value_name("PROJECT"))))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        init(&db, matches)?;
    }
    if let Some(matches) = matches.subcommand_matches("remotes") {
        if let Some(matches) = matches.subcommand_matches("add") {
            remotes_add(&db, matches)?;
        }
        if let Some(matches) = matches.subcommand_matches("list") {
            remotes_list(&db, matches)?;
        }
        if let Some(matches) = matches.subcommand_matches("update") {
            remotes_update(&db, matches)?;
        }
        if let Some(matches) = matches.subcommand_matches("delete") {
            remotes_delete(&db, matches)?;
        }
    }
    if let Some(matches) = matches.subcommand_matches("sync") {
        if let Some(matches) = matches.subcommand_matches("up") {
            sync_up(&db, Some(matches), None, None)?;
        }
        if let Some(matches) = matches.subcommand_matches("down") {
            sync_down(&db, matches)?;
        }
    }
    if let Some(matches) = matches.subcommand_matches("server") {
        if let Some(matches) = matches.subcommand_matches("start") {
            server_start(&db, matches, 13931, false, ||{})?;
        }
        if let Some(matches) = matches.subcommand_matches("status") {
            server_status(&db, matches)?;
        }
    }
    return Ok(());
}

fn init_database() -> Result<Option<Connection>, Box<dyn Error>> {
    let sql = vec![
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT UNIQUE NOT NULL,
            alias TEXT UNIQUE,
            created_date TIMESTAMP NOT NULL DEFAULT (datetime('now', 'localtime'))
        )",
        "CREATE TABLE IF NOT EXISTS remotes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name VARCHAR(32) UNIQUE NOT NULL,
            url TEXT NOT NULL,
            created_date TIMESTAMP NOT NULL DEFAULT (datetime('now', 'localtime')),
            last_modified TIMESTAMP NOT NULL DEFAULT (datetime('now', 'localtime'))
        )",
        "CREATE TABLE IF NOT EXISTS backups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project INTEGER NOT NULL,
            created_date TIMESTAMP NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY(project) REFERENCES projects(id)
        )"
    ];
    if let Some(home_dir) = dirs::home_dir() {
        let mut nick_home = String::new();
        nick_home.push_str(home_dir.to_str().unwrap());
        nick_home.push_str("/.nick");
        fs::create_dir(&nick_home).unwrap_or(());
        fs::create_dir(format!("{}/tmp", nick_home)).unwrap_or(());

        let mut db_path_str = String::new();
        db_path_str.push_str(nick_home.as_str());
        db_path_str.push_str("/nick.db");

        let db_path = Path::new(db_path_str.as_str());
        let db = Connection::open(db_path)?;
        for i in 0..sql.len() {
            db.execute(sql[i], params![])?;
        }

        return Ok(Some(db));
    }

    Ok(None)
}
fn init(db: &Connection, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut path = std::env::current_dir()?;
    if matches.is_present("path") {
        if let Some(provided_path) = matches.value_of("path") {
            path = Path::new(provided_path).to_path_buf();
        }
    }
    let mut project_alias = "";
    let mut path_str_repl = String::new();
    if matches.is_present("name") {
        project_alias = matches.value_of("name").unwrap();
    }
    else {
        let path_str = path.to_str().unwrap().to_string();
        path_str_repl = path_str.replace("\\", "/");
        let path_split: Vec<&str> = path_str_repl.split("/").collect();
        project_alias = path_split[path_split.len() - 1]
    }

    let full_path = fs::canonicalize(&path)?;
    let full_path = full_path.to_str().unwrap();
    let sql = "INSERT INTO projects (path, alias) VALUES (?1, ?2)";
    if let Err(_e) = db.execute(sql, params![full_path, project_alias]) {
        println!("ERROR: Project already exists at {}", full_path);
        return Ok(());
    }
    println!("Created project at {}", full_path);

    Ok(())
}
fn remotes_add(db: &Connection, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let name;
    let url;
    match matches.value_of("name") {
        Some(name_local) => {
            name = name_local;
        },
        None => {
            println!("ERROR: Remote must have a name");
            return Ok(())
        }
    }
    match matches.value_of("url") {
        Some(url_local) => {
            url = url_local;
        },
        None => {
            println!("ERROR: Remote must have a url");
            return Ok(())
        }
    }

    let sql = "SELECT COUNT(1) AS name_count FROM remotes WHERE name = ?1";
    let mut stmt = db.prepare(sql)?;
    let result: i32 = stmt.query_row(params![name], |row| {
        let rs: i32 = row.get("name_count")?;
        return Ok(rs);
    })?;

    if result <= 0 {
        let sql = "INSERT INTO remotes (name, url) VALUES (?1, ?2)";
        match db.execute(sql, params![name, url]) {
            Err(e) => println!("ERROR: {}", e.to_string().as_str()),
            Ok(_) => println!("Created remote '{}'", name)
        }
    }
    else {
        println!("ERROR: A remote with the name '{}' already exists", name);
    }

    Ok(())
}
fn remotes_list(db: &Connection, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let sql = "SELECT name, url FROM remotes";
    let mut stmt = db.prepare(sql)?;
    let mut results = stmt.query(params![])?;

    let mut names = vec![];
    let mut urls = vec![];
    let mut max_name_len = 0;
    let mut max_url_len = 0;
    while let Some(row) = results.next()? {
        let name: String = row.get("name")?;
        let url: String = row.get("url")?;
        if name.len() > max_name_len {
            max_name_len = name.len();
        }
        if url.len() > max_url_len {
            max_url_len = url.len();
        }
        names.push(name);
        urls.push(url);
    }

    fn repeat_char(c: &str, n: usize) -> String {
        let mut out = String::new();
        for _i in 0..n {
            out.push_str(c);
        }
        return out;
    }

    if !matches.is_present("simple") {
        if names.len() == 0 {
            println!("No remotes configured");
        }
        else {
            println!("name{}url", repeat_char(" ", max_name_len));
            println!("{}", repeat_char("-", max_name_len + 4 + max_url_len));
            for i in 0..names.len() {
                let name = &names[i];
                let url = &urls[i];
                println!("{}{}{}", name, repeat_char(" ", max_name_len - name.len() + 4), url);
            }
        }
    }
    else {
        for i in 0..names.len() {
            let name = &names[i];
            let url = &urls[i];
            println!("{},{}", name, url);
        }
    }

    Ok(())
}
fn remotes_update(db: &Connection, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let name;
    match matches.value_of("name") {
        Some(name_local) => name = name_local,
        None => {
            println!("ERROR: Remote name must be provided");
            return Ok(());
        }
    }
    let new_name = matches.value_of("new_name").unwrap_or(name);
    let new_url = matches.value_of("new_url").unwrap_or("");

    let mut stmt = db.prepare("SELECT url FROM remotes WHERE name = ?1")?;
    let mut url: String;
    match stmt.query_row(params![name], |row| {
        let u: String = row.get("url")?;
        return Ok(u);
    }) {
        Ok(url_local) => url = url_local,
        Err(_e) => {
            println!("ERROR: Remote '{}' does not exist", name);
            return Ok(());
        }
    };

    let sql = if new_url == "" { "UPDATE remotes SET name = ?1 WHERE name = ?2" } else { "UPDATE remotes SET name = ?1, url = ?2 WHERE name = ?3" };
    if new_url == "" {
        db.execute(sql, params![new_name, name])?;
    }
    else {
        db.execute(sql, params![new_name, new_url, name])?;
    }

    if new_name == name && (new_url == "" || new_url == url) {
        println!("No updates were made");
    }
    if new_name != name {
        println!("{} --> {}", name, new_name);
    }
    if new_url != url && new_url != "" {
        println!("{} --> {}", url, new_url);
    }

    Ok(())
}
fn remotes_delete(db: &Connection, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let name = matches.value_of("name").unwrap_or("");
    if name == "" {
        println!("ERROR: Remote name must be provided");
    }

    if let Ok(result) = db.execute("DELETE FROM remotes WHERE name = ?1", params![name]) {
        if result == 0 {
            println!("ERROR: Remote '{}' does not exist", name);
        }
        else {
            println!("Deleted remote '{}'", name);
        }
    }

    Ok(())
}
fn sync_up(db: &Connection, matches: Option<&ArgMatches>, project: Option<String>, address: Option<String>) -> Result<(), Box<dyn Error>> {
    let filename = format!("{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis());
    let tmp_file = format!("{}/{}.tar.gz", get_temp_path(), filename);

    let mut project_root: String = get_current_project("path", db, false)?;
    let mut project_name = get_current_project("name", db, true)?;
    let mut explicit = false;
    if let Some(matches) = matches {
        if matches.is_present("project") {
            project_name = matches.value_of("project").unwrap().to_string();
            let mut stmt = db.prepare("SELECT path FROM projects WHERE alias = ?1")?;
            match stmt.query_row(params![project_name], |row| {
                let tmp: String = row.get("path")?;
                return Ok(tmp);
            }) {
                Ok(project_root_local) => {
                    project_root = project_root_local;
                },
                Err(_) => {
                    println!("ERROR: Project '{}' does not exist", project_name);
                }
            };
        }
    }
    if let Some(project) = project {
        project_name = project;
        let mut stmt = db.prepare("SELECT path FROM projects WHERE alias = ?1")?;
        match stmt.query_row(params![project_name], |row| {
            let tmp: String = row.get("path")?;
            return Ok(tmp);
        }) {
            Ok(project_root_local) => {
                explicit = true;
                project_root = project_root_local;
            },
            Err(_) => {
                println!("ERROR: Project '{}' does not exist", project_name);
            }
        };
    }

    println!("Compressing project '{}'", project_name);
    {
        let mut archive = File::create(tmp_file.clone())?;
        let encoder = GzEncoder::new(&archive, Compression::default());
        let mut tar = tar::Builder::new(encoder);
        tar.append_dir_all(".", project_root.as_str())?;
        tar.finish()?;
    }

//    return Ok(());

    let mut remote = "";
    if !explicit {
        remote = matches.unwrap().value_of("remote").unwrap();
        println!("Connecting to '{}'", remote);
    }
    let mut port = None;
    let mut address_calc = None;
    if explicit {
        port = Some(13932);
        address_calc = address;
    }

    match server_remote_connect(db, remote.to_string(), port, address_calc) {
        Err(e) => {
            println!("ERROR: Could not connect to remote '{}' ({})", remote, get_remote_url_by_name(db, remote)?);
            println!("{}", e);
        }
        Ok(mut stream) => {
            println!("Syncing '{}' up...", project_name);
            let mut archive_file = File::open(&tmp_file)?;
            let archive_meta = fs::metadata(&tmp_file)?;
            let filesize = archive_file.metadata()?.len();

            stream.write_all(&mut [1u8]);

            let mut project_name_arr = project_name.as_bytes();

            let mut project_name_len_arr = &mut [0u8; 4];
            project_name_len_arr.write_with::<u32>(&mut 0, project_name_arr.len() as u32, BE).unwrap();
            stream.write_all(project_name_len_arr);
            stream.write_all(project_name_arr);

            let mut filesize_arr = &mut [0u8; 8];
            let mut offset = &mut 0;
            filesize_arr.write_with::<u64>(offset, filesize, BE).unwrap();
            stream.write_all(filesize_arr);

            let digest = sha256_digest(File::open(&tmp_file)?)?;
            let mut digest = digest.as_ref();
            stream.write_all(&mut digest);
            stream.flush();

            let mut buf = [0u8; 1024];

            loop {
                let count = archive_file.read(&mut buf)?;
                if count == 0 {
                    break;
                }
                stream.write(&mut buf[..count])?;
            }
            stream.flush();

            let response = &mut [0u8];
            stream.read_exact(response);
            if response[0] == 2 {
                println!("ERROR: Invalid checksum from remote");
                return Ok(());
            }
            else if response[0] != 0 {
                println!("ERROR: Remote responded with unknown error code");
                return Ok(());
            }

            println!("Sync completed");
        },
    }

    fs::remove_file(tmp_file);

    Ok(())
}
fn sync_down(db: &Connection, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut project_root: String = get_current_project("path", db, false)?;
    let mut project_name = get_current_project("name", db, true)?;
    if matches.is_present("project") {
        project_name = matches.value_of("project").unwrap().to_string();
        let mut stmt = db.prepare("SELECT path FROM projects WHERE alias = ?1")?;
        match stmt.query_row(params![project_name], |row| {
            let tmp: String = row.get("path")?;
            return Ok(tmp);
        }) {
            Ok(project_root_local) => {
                project_root = project_root_local;
            },
            Err(_) => {
                println!("ERROR: Project '{}' does not exist", project_name);
            }
        };
    }
    println!("Syncing '{}' down...", project_name);
    let remote = matches.value_of("remote").unwrap();
    server_start(db, matches, 13932, true, || {
        match server_remote_connect(db, remote.to_string(), None, None) {
            Err(_) => {
                println!("ERROR: Could not connect to remote '{}' ({})", remote, get_remote_url_by_name(db, remote).unwrap());
            }
            Ok(mut stream) => {
                stream.write_all(&mut [2u8]);

                let mut project_name_arr = project_name.as_bytes();

                let mut project_name_len_arr = &mut [0u8; 4];
                project_name_len_arr.write_with::<u32>(&mut 0, project_name_arr.len() as u32, BE).unwrap();
                stream.write_all(project_name_len_arr);
                stream.write_all(project_name_arr);
            },
        }
    })?;

    Ok(())
}
fn server_start<F: FnOnce()>(db: &Connection, matches: &ArgMatches, listen_port: u16, single_client: bool, start_callback: F) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", listen_port)).unwrap();
    start_callback();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let addr = stream.peer_addr().unwrap();
                if !single_client {
                    println!("{} established", addr);
                }
                let handle = thread::spawn(move|| {
                    let db_local = Connection::open(format!("{}/.nick/nick.db", dirs::home_dir().unwrap().to_str().unwrap())).unwrap();
                    handle_server_client(&db_local, stream, &addr);

                    if single_client {
                        println!("Sync completed");
                    }
                    else {
                        println!("{} closed", addr);
                    }
                });
                if single_client {
                    handle.join();
                    std::process::exit(0);
                }
            }
            Err(e) => {
                println!("ERROR: Connection to client failed: {}", e);
            }
        }
    }

    Ok(())
}
fn server_status(db: &Connection, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let remote = matches.value_of("remote").unwrap_or("");
    if remote == "" {
        println!("ERROR: Remote name must be provided");
        return Ok(());
    }
    let url = get_remote_url_by_name(db, remote)?;
    match TcpStream::connect(format!("{}:13931", url)) {
        Ok(mut stream) => {
            let mut command = [0 as u8];
            stream.write_all(&mut command);
            let mut response = [0 as u8];

            match stream.read_exact(&mut response) {
                Ok(_) => {
                    println!("Status: ON");
                    return Ok(());
                },
                Err(e) => {
                    println!("Status: OFF ({})", e.description());
                    return Ok(());
                }
            }
        },
        Err(e) => {
            println!("Status: OFF ({})", e.description());
            return Ok(());
        }
    }
}


/*
Server command reference:
Receive:
0 -> Status
1 -> Sync up (client perspective)
2 -> Sync down (client perspective)
Send:
0 -> Ok
1 -> Error: Unknown command
2 -> Error: Bad checksum
*/
fn handle_server_stream_close(mut stream: TcpStream) {
    stream.shutdown(Shutdown::Both).unwrap();
}
fn handle_server_client(db: &Connection, mut stream: TcpStream, address: &SocketAddr) {
    let mut command = [0 as u8; 1];
    match stream.read_exact(&mut command) {
        Ok(_) => {
//            println!("COMMAND: {:?}", command);
            let result = match command[0] {
                0 => handle_server_status(db, stream),
                1 => handle_server_sync_up(db, stream),
                2 => handle_server_sync_down(db, stream, address),
                _ => {
                    stream.write(&[1 as u8]).unwrap();
                    Ok(())
                }
            };
            if let Err(e) = result {
                println!("ERROR: {}", e.to_string());
            }
        },
        Err(e) => {
            println!("ERROR: {}", e.to_string());
            handle_server_stream_close(stream);
            return ();
        }
    }
}
fn handle_server_status(db: &Connection, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    stream.write(&[0 as u8]).unwrap();
    handle_server_stream_close(stream);

    Ok(())
}
fn handle_server_sync_up(db: &Connection, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let filename = format!("{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis());
    let tmp_file = format!("{}/{}.tar.gz", get_temp_path(), filename);
    let tmp_dir = get_temp_path();

    let project_name_len_arr = &mut [0u8; 4];
    stream.read_exact(project_name_len_arr)?;
    let project_name_len = project_name_len_arr.read_with::<u32>(&mut 0, BE).unwrap();

    let mut project_name_arr = vec![0u8; project_name_len as usize];
    let project_name_arr = project_name_arr.as_mut_slice();
    stream.read_exact(project_name_arr)?;
    let project_name = std::str::from_utf8(project_name_arr)?;

    let filesize_arr = &mut [0u8; 8];
    stream.read_exact(filesize_arr)?;
    let filesize = filesize_arr.read_with::<u64>(&mut 0, BE).unwrap();

    let mut hash_arr = [0u8; 32];
    stream.read_exact(&mut hash_arr)?;

    {
        let mut out_file = File::create(&tmp_file)?;
        let mut count = filesize;
        let mut buf = [0u8; 1024];
        let mut context = Context::new(&SHA256);
        loop {
            let read = stream.read(&mut buf)?;
            count -= read as u64;
            let contents = &buf[..read];
            context.update(contents);
            out_file.write(contents);
            if count == 0 {
                break;
            }
        }
        let digest = context.finish();
        let mut calculated_hash = digest.as_ref();
        if !calculated_hash.eq(&hash_arr) {
            println!("WARNING: Bad checksum");
            stream.write(&[2u8]);
            fs::remove_file(&tmp_file)?;
            stream.flush();
            handle_server_stream_close(stream);
            return Ok(());
        }
    }

    let mut stmt = db.prepare("SELECT id, path FROM projects WHERE alias = ?1")?;
    let mut project_id: u32 = 0;
    let mut project_path: String = String::new();
    stmt.query_row(params![project_name], |row| {
        project_id = row.get("id")?;
        project_path = row.get("path")?;
        return Ok(());
    })?;
    db.execute("INSERT INTO backups (project) VALUES (?1)", params![project_id]);
    let mut backup_id: u32 = 0;
    db.query_row("SELECT id FROM backups WHERE project = ?1 ORDER BY created_date DESC LIMIT 1", params![project_id], |row| {
        backup_id = row.get("id")?;
        return Ok(());
    })?;

    let backup_path = format!("{}/backups/{}", get_data_path(), project_id);
    fs::create_dir_all(&backup_path).unwrap_or(());
    let backup_file_path = format!("{}/{}.tar.gz", &backup_path, backup_id);

    let mut backup_archive = File::create(&backup_file_path)?;
    let encoder = GzEncoder::new(&backup_archive, Compression::default());
    let mut backup_tar = tar::Builder::new(encoder);
    backup_tar.append_dir_all(".", project_path.as_str())?;
    backup_tar.finish()?;

    let mut stmt = db.prepare("SELECT id FROM backups WHERE project = ?1 ORDER BY created_date DESC")?;
    let mut backups_results = stmt.query(params![project_id])?;
    let mut backup_count = 0;
    while let Some(backup_result) = backups_results.next()? {
        backup_count += 1;
        if backup_count > 5 {
            let backup_id_iter: u32 = backup_result.get("id")?;
            db.execute("DELETE FROM backups WHERE id = ?1", params![backup_id_iter])?;
            fs::remove_file(format!("{}/{}.tar.gz", &backup_path, backup_id_iter)).unwrap_or(());
        }
    }

    fs::remove_dir_all(&project_path).unwrap_or(());
    fs::create_dir_all(&project_path).unwrap_or(());

//    println!("TMP FILE: {}", tmp_file);
//    println!("PROJECT PATH: {}", project_path);

    let out_file2 = File::open(&tmp_file)?;
    let decoder = GzDecoder::new(out_file2);
    let mut archive = Archive::new(decoder);
    archive.unpack(&project_path)?;

    stream.write(&[0u8]);
    stream.flush()?;

    Ok(())
}
fn handle_server_sync_down(db: &Connection, mut stream: TcpStream, address: &SocketAddr) -> Result<(), Box<dyn Error>> {
    let project_name_len_arr = &mut [0u8; 4];
    stream.read_exact(project_name_len_arr)?;
    let project_name_len = project_name_len_arr.read_with::<u32>(&mut 0, BE).unwrap();

    let mut project_name_arr = vec![0u8; project_name_len as usize];
    let project_name_arr = project_name_arr.as_mut_slice();
    stream.read_exact(project_name_arr)?;
    let project_name = std::str::from_utf8(project_name_arr)?;

    sync_up(db, None, Some(project_name.to_string()), Some(format!("{}", address.ip())));

    Ok(())
}
fn get_current_project(prop: &str, db: &Connection, suppress_warnings: bool) -> Result<String, Box<dyn Error>> {
    let current_dir = std::env::current_dir()?;
    let current_dir = fs::canonicalize(&current_dir)?;
    let current_dir_str = current_dir.to_str().unwrap().to_string();
    let sql = "SELECT id, path, alias FROM projects ORDER BY LENGTH(path) ASC";
    let mut stmt = db.prepare(sql)?;
    let mut results = stmt.query(params![])?;

    let mut parent_paths = vec![];
    let mut lowest_parent_path = String::new();
    let mut lowest_parent_name = String::new();
    let mut lowest_parent_id: i64 = -1;
    while let Some(row) = results.next()? {
        let project_path: String = row.get("path")?;
        let project_name: String = row.get("alias")?;
        let project_id: i64 = row.get("id")?;
        if current_dir_str.contains(project_path.as_str()) {
            if lowest_parent_path.len() < project_path.len() {
                lowest_parent_path = project_path.clone();
                lowest_parent_name = project_name.clone();
                lowest_parent_id = project_id;
            }
            parent_paths.push(project_path.clone());
        }
    }

    if !suppress_warnings && parent_paths.len() > 1 {
        println!("WARNING: Multiple parent projects:");
        for i in 0..parent_paths.len() {
            print!("\t{}. {}", i+1, parent_paths[i]);
            if parent_paths[i].eq(&lowest_parent_path) {
                print!("  <-- Using this one");
            }
            println!();
        }
    }

    match prop {
        "path" => return Ok(lowest_parent_path),
        "name" => return Ok(lowest_parent_name),
        _ => return Ok(lowest_parent_id.to_string())
    }
}
fn server_remote_connect(db: &Connection, remote: String, port: Option<u16>, address: Option<String>) -> Result<TcpStream, Box<dyn Error>> {
    let mut url = String::new();
    let mut real_port = 13931;
    match address {
        Some(addr) => {
            url = addr;
            real_port = port.unwrap();
        },
        None => {
            url = get_remote_url_by_name(db, remote.as_str())?;
        }
    }
    match TcpStream::connect(format!("{}:{}", url, real_port)) {
        Ok(mut stream) => {
            return Ok(stream);
        },
        Err(e) => {
            return Err(Box::new(e));
        }
    }
}
fn get_remote_url_by_name(db: &Connection, name: &str) -> Result<String, Box<dyn Error>> {
    let sql = "SELECT url FROM remotes WHERE name = ?1";
    let mut stmt = db.prepare(sql)?;
    return match stmt.query_row(params![name], |row| {
        let u: String = row.get("url")?;
        return Ok(u);
    }) {
        Ok(url) => Ok(url),
        Err(_) => Ok("".to_string())
    }
}
fn get_data_path() -> String {
    let home = dirs::home_dir().unwrap();
    return format!("{}/.nick", home.to_str().unwrap());
}
fn get_temp_path() -> String {
    return format!("{}/tmp", get_data_path());
}
fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest, Box<dyn Error>> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}
