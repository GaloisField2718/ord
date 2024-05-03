use bitcoin::{OutPoint, ScriptBuf, Transaction, TxIn, TxOut};
use bitcoin::consensus::serialize;
use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Mint {
  #[clap(long, help = "Use <FEE_RATE> sats/vbyte for mint transaction.")]
  pub fee_rate: FeeRate,
  #[clap(long, help = "Mint <RUNE>. May contain `.` or `•`as spacers.")]
  pub rune: SpacedRune,
  #[clap(long, help = "Add <UTXO> (TxID:vout) to choose specific utxo.")]
  pub utxo: Option<String>,
  #[clap(
    long,
    help = "Include <AMOUNT> postage with mint output. [default: 10000sat]"
  )]
  pub postage: Option<Amount>,
  #[clap(long, help = "Send minted runes to <DESTINATION>.")]
  pub destination: Option<Address<NetworkUnchecked>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output{
  pub rune: SpacedRune,
  pub pile: Pile,
  pub mint: Txid,
  pub rawtx: String,
}

impl Mint {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      wallet.has_rune_index(),
      "`ord wallet mint` requires index created with `--index-runes` flag",
    );

    let rune = self.rune.rune;

    let bitcoin_client = wallet.bitcoin_client();

    let block_height = bitcoin_client.get_block_count()?;

    let Some((id, rune_entry, _)) = wallet.get_rune(rune)? else {
      bail!("rune {rune} has not been etched");
    };

    let postage = self.postage.unwrap_or(TARGET_POSTAGE);

    let amount = rune_entry
      .mintable(block_height + 1)
      .map_err(|err| anyhow!("rune {rune} {err}"))?;

    let chain = wallet.chain();

    let destination = match self.destination {
      Some(destination) => destination.require_network(chain.network())?,
      None => wallet.get_change_address()?,
    };

    ensure!(
      destination.script_pubkey().dust_value() < postage,
      "postage below dust limit of {}sat",
      destination.script_pubkey().dust_value().to_sat()
    );

    let runestone = Runestone {
      mint: Some(id),
      ..default()
    };

    let script_pubkey = runestone.encipher();

    ensure!(
      script_pubkey.len() <= 82,
      "runestone greater than maximum OP_RETURN size: {} > 82",
      script_pubkey.len()
    );
    
           let mut inputs = Vec::new();

        if let Some(utxo) = self.utxo {
            // Parse the utxo string into txid and vout
            let (txid, vout) = parse_utxo(&utxo)?;

            // Construct a TxIn using the parsed txid and vout
            let txin = TxIn {
                previous_output: OutPoint {
                    txid: bitcoin::Txid::from_str(&txid)?,
                    vout,
                },
                script_sig: ScriptBuf::default(), // or Script::new() if you prefer
                sequence: bitcoin::Sequence::ENABLE_RBF_NO_LOCKTIME, // Wrap the integer value with bitcoin::Sequence
                witness: Witness::default(), // Create an empty Witness
            };

            inputs.push(txin);
        } 

    let unfunded_transaction = Transaction {
      version: 2,
      lock_time: LockTime::ZERO,
      input: inputs,
      output: vec![
        TxOut {
          script_pubkey,
          value: 0,
        },
        TxOut {
          script_pubkey: destination.script_pubkey(),
          value: postage.to_sat(),
        },
      ],
    };

    wallet.lock_non_cardinal_outputs()?;

    let unsigned_transaction =
      fund_raw_transaction(bitcoin_client, self.fee_rate, &unfunded_transaction)?;

    let signed_transaction = bitcoin_client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let signed_transaction = consensus::encode::deserialize(&signed_transaction)?;

    assert_eq!(
      Runestone::decipher(&signed_transaction),
      Some(Artifact::Runestone(runestone)),
    );
    // Convert the transaction to bytes
    let signed_transaction_bytes = serialize(&signed_transaction);
    
    let rawtx = hex::encode(&signed_transaction_bytes);
    let txid = signed_transaction.txid();
    
    // Encode the bytes as a hexadecimal string
    //println!("Signed Transaction: {}", hex::encode(&signed_transaction_bytes));    
    //let transaction = bitcoin_client.send_raw_transaction(&signed_transaction)?;
    //println!("Transaction ID {}", signed_transaction.txid());

    Ok(Some(Box::new(Output {
      rune: self.rune,
      pile: Pile {
        amount,
        divisibility: rune_entry.divisibility,
        symbol: rune_entry.symbol,
      },
      mint: txid,
      rawtx: rawtx,
    })))
  }

}

// Function to parse utxo string into txid and vout
fn parse_utxo(utxo: &str) -> Result<(String, u32), anyhow::Error> {
    let parts: Vec<&str> = utxo.split(':').collect();
    if parts.len() != 2 {
        bail!("Invalid UTXO format. Expected format: txid:vout");
    }

    let txid = parts[0].to_string();
    let vout = parts[1].parse()?;
    Ok((txid, vout))
}
