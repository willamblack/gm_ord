use {super::*, fee_rate::FeeRate};

#[derive(Debug, Parser)]
pub(crate) struct Preview {
  #[clap(flatten)]
  server: super::server::Server,
  inscriptions: Vec<PathBuf>,
}

struct KillOnDrop(process::Child);

impl Drop for KillOnDrop {
  fn drop(&mut self) {
    self.0.kill().unwrap()
  }
}

impl Preview {
  pub(crate) fn run(self) -> Result {
    let tmpdir = TempDir::new()?;

    let rpc_port = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();

    let bitcoin_data_dir = tmpdir.path().join("bitcoin");

    fs::create_dir(&bitcoin_data_dir)?;

    let _bitcoind = KillOnDrop(
      Command::new("bitcoind")
        .arg({
          let mut arg = OsString::from("-datadir=");
          arg.push(&bitcoin_data_dir);
          arg
        })
        .arg("-regtest")
        .arg("-txindex")
        .arg("-listen=0")
        .arg(format!("-rpcport={rpc_port}"))
        .spawn()
        .context("failed to spawn `bitcoind`")?,
    );

    let options = Options {
      chain_argument: Chain::Regtest,
      bitcoin_data_dir: Some(bitcoin_data_dir),
      data_dir: Some(tmpdir.path().into()),
      rpc_url: Some(format!("127.0.0.1:{rpc_port}")),
      index_sats: true,
      index_utxos: false,
      ..Options::default()
    };

    for attempt in 0.. {
      if options.bitcoin_rpc_client().is_ok() {
        break;
      }

      if attempt == 100 {
        panic!("Bitcoin Core RPC did not respond");
      }

      thread::sleep(Duration::from_millis(50));
    }

    super::wallet::Wallet::Create(super::wallet::create::Create {
      passphrase: "".into(),
    })
    .run(options.clone())?;

    let rpc_client = options.bitcoin_rpc_client_for_wallet_command(false)?;

    let address = rpc_client
      .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))?
      .require_network(Network::Regtest)?;

    rpc_client.generate_to_address(101, &address)?;

    Arguments {
      options: options.clone(),
      subcommand: Subcommand::Wallet(super::wallet::Wallet::Inscribe(
        super::wallet::inscribe::Inscribe {
          fee_rate: FeeRate::try_from(1.0).unwrap(),
          commit_fee_rate: None,
          files: self.inscriptions,
          no_backup: true,
          no_broadcast: false,
          wait_after_commit: false,
          satpoint: None,
          utxo: Vec::new(),
          coin_control: false,
          dry_run: false,
          dump: false,
          no_limit: false,
          destination: Vec::new(),
          alignment: None,
          cursed_destination: None,
          cursed_utxo: None,
          cursed: false,
          change: None,
          postage: Some(TransactionBuilder::DEFAULT_TARGET_POSTAGE),
          max_inputs: None,
          csv: None,
          cursed66: false,
          no_signature: false,
          allow_reinscribe: false,
          ignore_utxo_inscriptions: false,
          single_key: false,
          allow_reveal_rbf: false,
          unfunded_reveal: false,
        },
      )),
    }
    .run()?;

    rpc_client.generate_to_address(1, &address)?;

    Arguments {
      options,
      subcommand: Subcommand::Server(self.server),
    }
    .run()?;

    Ok(())
  }
}
