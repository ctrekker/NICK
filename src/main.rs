// NICK - New Integrated Context Karyover
//   The acronym sucks, and so does the workflow

use clap::{App, SubCommand, Arg, ArgMatches};
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;
use std::error::Error;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn Error>> {
    let db: Connection = (init_database()?).unwrap();
    let matches = App::new("NICK")
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
            .arg(Arg::with_name("alias")
                .short("a")
                .long("alias")
                .takes_value(true)
                .value_name("ALIAS")
                .help("Aliased project name. Used for remote access to project without knowing local path")))
        .subcommand(SubCommand::with_name("serve")
            .about("Serves project(s) code on server")
            .arg(Arg::with_name("global")
                .short("g")
                .long("global")
                .help("Serves all initialized projects on local machine. Mutually exclusive with project"))
            .arg(Arg::with_name("project")
                .short("p")
                .long("project")
                .help("Serves project based on name. Mutually exclusive with global")))
        .subcommand(SubCommand::with_name("remotes")
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
                    .help("Name of the remote. Used for identification only")
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
                .about("Deletes a remote")))
        .subcommand(SubCommand::with_name("sync")
            .about("Syncs current code repo with remotes. Defaults to all remotes")
            .subcommand(SubCommand::with_name("up")
                .about("Syncs local code up to remote code. Overrides remote code"))
            .subcommand(SubCommand::with_name("down")
                .about("Syncs remote code down to local code. Overrides local code")))
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
        )"
    ];
    if let Some(home_dir) = dirs::home_dir() {
        let mut nick_home = String::new();
        nick_home.push_str(home_dir.to_str().unwrap());
        nick_home.push_str("/.nick");
        fs::create_dir(&nick_home).unwrap_or(());

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

    let full_path = fs::canonicalize(&path)?;
    let full_path = full_path.to_str().unwrap();
    let sql = "INSERT INTO projects (path, alias) VALUES (?1, ?2)";
    if let Err(e) = db.execute(sql, params![full_path, matches.value_of("alias")]) {
        println!("ERROR: Project already exists at {}", full_path);
        return Ok(());
    }
    println!("Created project at {}", full_path);

    Ok(())
}
fn remotes_add(db: &Connection, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut name = "";
    let mut url = "";
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
        for i in 0..n {
            out.push_str(c);
        }
        return out;
    }

    if !matches.is_present("simple") {
        println!("name{}url", repeat_char(" ", max_name_len));
        println!("{}", repeat_char("-", max_name_len + 4 + max_url_len));
        for i in 0..names.len() {
            let name = &names[i];
            let url = &urls[i];
            println!("{}{}{}", name, repeat_char(" ", max_name_len - name.len() + 4), url);
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
    let mut name = "";
    match matches.value_of("name") {
        Some(name_local) => name = name_local,
        None => {
            println!("ERROR: Remote name must be provided");
            return Ok(());
        }
    }
    let mut new_name = matches.value_of("new_name").unwrap_or(name);
    let mut new_url = matches.value_of("new_url").unwrap_or("");

    let mut stmt = db.prepare("SELECT url FROM remotes WHERE name = ?1")?;
    let mut url: String = String::new();
    match stmt.query_row(params![name], |row| {
        let u: String = row.get("url")?;
        return Ok(u);
    }) {
        Ok(url_local) => url = url_local,
        Err(e) => {
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


    Ok(())
}


fn get_current_project_id(db: &Connection, suppress_warnings: bool) -> Result<i64, Box<dyn Error>> {
    let current_dir = std::env::current_dir()?;
    let current_dir = fs::canonicalize(&current_dir)?;
    let current_dir_str = current_dir.to_str().unwrap().to_string();
    let sql = "SELECT id, path, alias FROM projects ORDER BY LENGTH(path) ASC";
    let mut stmt = db.prepare(sql)?;
    let mut results = stmt.query(params![])?;

    let mut parent_paths = vec![];
    let mut lowest_parent_path = String::new();
    let mut lowest_parent_id: i64 = -1;
    while let Some(row) = results.next()? {
        let project_path: String = row.get("path")?;
        let project_id: i64 = row.get("id")?;
        if current_dir_str.contains(project_path.as_str()) {
            if lowest_parent_path.len() < project_path.len() {
                lowest_parent_path = project_path.clone();
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

    return Ok(lowest_parent_id);
}
