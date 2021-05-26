mod command;

use dialoguer::Confirm;
use std::collections::HashMap;
use reqwest::Error;
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::Method;
use structopt::StructOpt;
use command::Command;
use serde::{Deserialize};

use std::iter::Iterator;

#[macro_export]
macro_rules! die {
  () => ({ print!("\n"); std::process::exit(-1) });
  ($fmt:expr) => ({ print!(concat!($fmt, "\n")); std::process::exit(-1) });
  ($fmt:expr, $($arg:tt)*) => ({ print!(concat!($fmt, "\n"), $($arg)*); std::process::exit(-1) });
}

#[derive(Deserialize, Debug)]
struct DocumentResultChange {
  rev: String,
}

#[derive(Deserialize, Debug)]
struct DocumentResult {
  id: String,
  changes: Vec<DocumentResultChange>,
  deleted: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct Document {
  results: Vec<DocumentResult>,
}

fn request(url: String, method: Method, username: &String, password: &String, params: HashMap<String, Vec<String>>) -> Result<Response, Error> {
  // println!("Request: [ {} ] {}...", method, url);
  Client::builder()
    .build()
    .map(|client| match method {
      Method::GET => client.get(url),
      Method::POST => client.post(url),
      _ => die!("Unexpected method (both GET and POST supported)"),
    })
    .and_then(|req| req
      .basic_auth(username, Some(password))
      .json(&params)
      .send()
    )
}

fn get_database_list(base_url: &String, username: &String, password: &String) -> Result<Vec<String>, Error> {
  Ok(format!("{}/_all_dbs", base_url))
    .and_then(|url| request(url, Method::GET, username, password, HashMap::new()))
    .and_then(|res| res.json::<Vec<String>>())
}

fn get_document_list(base_url: &String, username: &String, password: &String, database: &String) -> Result<Document, Error> {
  Ok(format!("{}/{}/_changes", base_url, database))
    .and_then(|url| request(url, Method::GET, username, password, HashMap::new()))
    .and_then(|res| res.json::<Document>())
}

fn purge(base_url: &String, username: &String, password: &String, database: &String, document: &String, revisions: Vec<String>) -> Result<(), Error> {
  let mut params = HashMap::new();
  params.insert(String::from(document), revisions);
  Ok(format!("{}/{}/_purge", base_url, database))
    .and_then(|url| request(url, Method::POST, username, password, params))
    .map(|_| ())
}

fn compact(base_url: &String, username: &String, password: &String, database: &String) -> Result<(), Error> {
  Ok(format!("{}/{}/_compact", base_url, database))
    .and_then(|url| request(url, Method::GET, username, password, HashMap::new()))
    .map(|_| ())
}

fn view_cleanup(base_url: &String, username: &String, password: &String, database: &String) -> Result<(), Error> {
  Ok(format!("{}/{}/_view_cleanup", base_url, database))
    .and_then(|url| request(url, Method::GET, username, password, HashMap::new()))
    .map(|_| ())
}

fn main() {
  // extract CLI args
  let opt = Command::from_args();

  // show command line as json object
  if opt.debug {
    println!("{:?}", opt);
  }

  let base_url = format!("{}://{}:{}", opt.protocol, opt.host, opt.port);

  let username = opt.username;
  let password = opt.password;

  let force_yes = opt.force_yes;

  // todo check / entrypoint before processing

  opt.database
    .map(|databases| { println!("Specified database(s): "); databases})
    .or_else(|| { print!("No database(s) specified, retrieving all... "); None})
    .or_else(|| get_database_list(&base_url, &username, &password)
      .map(|databases| { println!("OK"); databases})
      .map_err(|err| die!("ERROR !\nCause: {:?}", err))
      .ok()
    ) // default value
    .map(|databases| { println!("List:\n - {}\n", databases.join("\n - ")); databases})
    // TODO do not miss pointer
    .filter(|_| force_yes || Confirm::new().with_prompt("WARNING ! Deleted documents stored in below databases will be purged ! Do you want to continue?").wait_for_newline(true).default(false).interact().unwrap())
    .unwrap_or_else(|| die!())
    .into_iter()
    .map(|database| { println!("Purging database [{}]", database); database })
    .map(|database| get_document_list(&base_url, &username, &password, &database)
      .map_err(|err| println!("ERROR !\nCause: {:?}", err))
      .map(|document| (database, document.results))
      .map(|(database, results)| results
        .into_iter()
        .filter(|result| result.deleted.is_some())
        .map(|result| (database.to_string(), result.id, result.changes))
        .map(|(database, id, revisions)| purge(&base_url, &username, &password, &database, &id, revisions.into_iter().map(|revision| revision.rev).collect::<Vec<String>>())
          .map_err(|err| { println!("ERROR !\nCause: {:?}", err); err })
        )
        .filter_map(Result::ok)
        .map(|_| database.to_string())
        .collect::<Vec<String>>()
      )
    )
    .filter_map(Result::ok)
    .map(|database| database.into_iter().next())
    .map(|database| { database.is_none().then(|| println!("No deleted documents in database\n")); database })
    .filter(Option::is_some)
    .map(Option::unwrap)
    .map(|database| Ok(())
      .and_then(|_| compact(&base_url, &username, &password, &database))
      .map(|_| { println!("Compacting database [{}]", database); () })
      .map_err(|err| { println!("ERROR !\nCause: {:?}", err); err })
      .and_then(|_| view_cleanup(&base_url, &username, &password, &database))
      .map(|_| { println!("ViewCleanup database [{}]\n", database); () })
      .map_err(|err| { println!("ERROR !\nCause: {:?}", err); err })
    )
    .filter_map(Result::ok)
    .for_each(drop);
}
