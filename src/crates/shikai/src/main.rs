use alloy_primitives::{hex::{self, FromHex}, keccak256, Address, FixedBytes, TxKind, U256};
use dotenv::from_filename;
use shikai::Shikai;

pub struct ClaimedExecution {
    pub tx_hash: FixedBytes<32>,
    pub receiver: Address,
    pub amount: U256,
}

impl ClaimedExecution {
    pub async fn verify(&self, shikai: &Shikai) -> bool {
        let tx = shikai.execution().tx(self.tx_hash).await.unwrap();

        if tx.value() != self.amount {
            return false;
        }

        if tx.to() != TxKind::Call(self.receiver) {
            return false;
        }

        true
    }

    pub fn id(&self) -> FixedBytes<32> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.receiver.as_slice());
        data.extend_from_slice(&self.amount.as_le_slice());
        FixedBytes::from(keccak256(data))
    }
}




#[tokio::main]
async fn main() {
    from_filename(".env.local").ok();

    let shikai = Shikai::new(
        std::env::var("EXECUTION_RPC_URL").unwrap(),
        std::env::var("BEACON_RPC_URL").unwrap(),
    );

    let claimed_executions = get_claimed_executions();
    let mut valid_executions = Vec::new();

    for claimed_execution in claimed_executions {
        let verified = claimed_execution.verify(&shikai).await;
        if verified {
            valid_executions.push(claimed_execution.id());
        }
    }

    println!("Valid Executions: {:?}", valid_executions);

} 

fn get_claimed_executions() -> Vec<ClaimedExecution> {
    let claimed_executions = vec![
        ClaimedExecution {
            tx_hash: FixedBytes::<32>::from_hex("0x47cea127fc2dcf17430191190d7edfb4ce971d82e8bef7a8ec866b66512e53c5").unwrap(),
            receiver: Address::from_hex("0x6c5aAE4622B835058A41879bA5e128019B9047d6").unwrap(),
            amount: U256::from_be_slice(hex::decode("0x012A02FE3A6448D800").unwrap().as_slice()),
        },
        ClaimedExecution {
            tx_hash: FixedBytes::<32>::from_hex("0x76f99c726031d88bc4a78e39fba1304d458587b8d018fe6660e4d6ff8b6e337a").unwrap(),
            receiver: Address::from_hex("0x327FDab86397F41Ed67df6419A5260865BE0523B").unwrap(),
            amount: U256::from_be_slice(hex::decode("0x038D7EA4C68000").unwrap().as_slice()),
        },
    ];

    claimed_executions
}