use super::*;

pub mod check_index;
pub mod compact;
pub mod decode;
pub mod epochs;
pub mod find;
mod index;
pub mod info;
pub mod inscriptions;
pub mod list;
pub mod parse;
mod preview;
mod server;
pub mod subsidy;
pub mod supply;
pub mod traits;
pub mod transfer;
pub mod wallet;

fn print_json(output: impl Serialize) -> Result {
  serde_json::to_writer_pretty(io::stdout(), &output)?;
  println!();
  Ok(())
}

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  #[clap(about = "Check whether the index file needs recovery without attempting recovery")]
  CheckIndex(check_index::CheckIndex),
  #[clap(about = "Compact the index file if possible")]
  Compact,
  #[clap(about = "Decode inscription data from a transaction output")]
  Decode(decode::Decode),
  #[clap(about = "List the first satoshis of each reward epoch")]
  Epochs,
  #[clap(about = "Run an explorer server populated with inscriptions")]
  Preview(preview::Preview),
  #[clap(about = "Find a satoshi's current location")]
  Find(find::Find),
  #[clap(subcommand, about = "Index commands")]
  Index(index::IndexSubcommand),
  #[clap(about = "Display index statistics")]
  Info(info::Info),
  #[clap(about = "List all inscriptions")]
  Inscriptions(inscriptions::Inscriptions),
  #[clap(about = "List the satoshis in an output")]
  List(list::List),
  #[clap(about = "Parse a satoshi from ordinal notation")]
  Parse(parse::Parse),
  #[clap(about = "Display information about a block's subsidy")]
  Subsidy(subsidy::Subsidy),
  #[clap(about = "Run the explorer server")]
  Server(server::Server),
  #[clap(about = "Display Bitcoin supply information")]
  Supply,
  #[clap(about = "Display satoshi traits")]
  Traits(traits::Traits),
  #[clap(about = "Modify transfer log table")]
  Transfer(transfer::Transfer),
  #[clap(subcommand, about = "Wallet commands")]
  Wallet(wallet::Wallet),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::CheckIndex(check_index) => check_index.run(options),
      Self::Compact => compact::run(options),
      Self::Decode(decode) => decode.run(options),
      Self::Epochs => epochs::run(),
      Self::Preview(preview) => preview.run(),
      Self::Find(find) => find.run(options),
      Self::Index(index) => index.run(options),
      Self::Info(info) => info.run(options),
      Self::Inscriptions(inscriptions) => inscriptions.run(options),
      Self::List(list) => list.run(options),
      Self::Parse(parse) => parse.run(),
      Self::Subsidy(subsidy) => subsidy.run(),
      Self::Server(server) => {
        let index = Arc::new(Index::open(&options)?);
        let handle = axum_server::Handle::new();
        LISTENERS.lock().unwrap().push(handle.clone());
        server.run(options, index, handle)
      }
      Self::Supply => supply::run(),
      Self::Traits(traits) => traits.run(),
      Self::Transfer(transfer) => transfer.run(options),
      Self::Wallet(wallet) => wallet.run(options),
    }
  }
}
