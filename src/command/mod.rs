use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(Debug, StructOpt)]
#[structopt(name = "CouchDB Purge CLI", about = "An awesome CLI to purge your CouchDB databases.", global_settings = &[AppSettings::DeriveDisplayOrder])]
pub struct Command {
  /// Activate debug mode
  #[structopt(short="D", long)]
  pub debug: bool,

  /// Couchdb protocol
  #[structopt(long, default_value = "http")]
  pub protocol: String,

  /// Couchdb host
  #[structopt(short, long, default_value = "localhost")]
  pub host: String,

  /// Couchdb port
  #[structopt(short, long, default_value = "5984")]
  pub port: u16,

  /// Couchdb database
  #[structopt(short, long)]
  pub database: Option<Vec<String>>,

  /// Couchdb username
  #[structopt(short, long)]
  pub username: String,

  /// Couchdb password
  #[structopt(long)]
  pub password: String,

  /// Force yes
  #[structopt(short="y", long)]
  pub force_yes: bool,
}
