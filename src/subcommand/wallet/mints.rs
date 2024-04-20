//use bitcoin::{OutPoint, ScriptBuf, Transaction, TxIn, TxOut};
//use bitcoin::consensus::serialize;
use super::*;
use crate::subcommand::wallet::mint::Mint;

#[derive(Debug, Parser)]
pub(crate) struct Mints {
  #[clap(long, help = "Use <FEE_RATE> sats/vbyte for mint transaction.")]
  fee_rate: FeeRate,
  #[clap(long, help = "Mint <RUNE>. May contain `.` or `•`as spacers.")]
  rune: SpacedRune,
  #[clap(long, help = "Add <UTXO> (TxID:vout) to choose specific utxo.")]
  utxo: String,
  #[clap(long, help="<NUMBER> of mints to do.")]
  n: i32,
  #[clap(
    long,
    help = "Include <AMOUNT> postage with mint output. [default: 10000sat]"
  )]
  postage: Option<Amount>,
  #[clap(long, help = "Send minted runes to <DESTINATION>.")]
  destination: Option<Address<NetworkUnchecked>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output{
  pub rune: SpacedRune,
  //pub pile: Pile,
  //pub mint: Txid,
}

//#[derive(Serialize, Deserialize, Debug)]
/*pub struct Utxo{*/
    /*pub txid: Txid,*/
    /*pub vout: i32,*/
/*}*/

impl Mints {
  
    pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
        ensure!(
            wallet.has_rune_index(),
            "Need to have index created with runes index stupid MONKEY!");
    
        // Capture values from the Mints struct
        let fee_rate = self.fee_rate;
        let rune = self.rune.clone(); // Clone the rune because we'll move it into Mint later
        let utxo = self.utxo.clone(); // Clone the utxo because we'll move it into Mint later
        let postage = Some(self.postage.unwrap_or(TARGET_POSTAGE)); // Wrap postage in Some
        let destination = self.destination.clone();        

        let mint = Mint {
            fee_rate,
            rune,
            utxo: Some(utxo),
            postage,
            destination,
    };

    // Call the run function on the Mint object
    let output = mint.run(wallet)?;
    println!("{}", output.ok());

    // Return the result or handle it further
    //Ok(())

    Ok(Some(Box::new(Output {
      rune: self.rune,
    })))

  }
}

// Function to parse utxo string into txid and vout
/*fn parse_utxo(utxo: &str) -> Result<(String, u32), anyhow::Error> {*/
    /*let parts: Vec<&str> = utxo.split(':').collect();*/
    /*if parts.len() != 2 {*/
        /*bail!("Invalid UTXO format. Expected format: txid:vout");*/
    /*}*/

    /*let txid = parts[0].to_string();*/
    /*let vout = parts[1].parse()?;*/
    /*Ok((txid, vout))*/
/*}*/
