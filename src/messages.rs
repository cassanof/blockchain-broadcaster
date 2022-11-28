use std::{fmt::Display, str::FromStr};

pub struct Move {
    pub from: String,
    pub amount: f64,
}

impl FromStr for Move {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split(',').collect::<Vec<&str>>();
        if split.len() != 2 {
            return Err("Move has more than two parts".to_string());
        }
        let from = split.get(0).unwrap().to_string();
        // check it's actually base64
        let _ = base64::decode(&from)
            .map_err(|_| format!("Recipient public key ({}) is not base64", from))?;

        // check it's right length
        if from.len() != 116 {
            return Err(format!("Recipient public key ({}) is an invalid key", from));
        }

        let amount = split.get(1).unwrap().to_string();
        let amount = match amount.parse::<f64>() {
            Ok(x) => x,
            Err(_) => return Err("Amount is not a number".to_string()),
        };

        // check that the amount is positive and not too big

        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }

        if amount == f64::INFINITY {
            return Err("Amount is too big".to_string());
        }

        Ok(Move { from, amount })
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // if the amount = Infinity, then we need to print float max value
        let amount = if self.amount == f64::INFINITY {
            f64::MAX
        } else {
            self.amount
        };

        write!(f, "{},{}", self.from, amount)
    }
}

pub struct Transaction {
    pub serial: u64,
    pub unique_string: String,
    pub sig: String,
    pub sender: String,
    pub moves: Vec<Move>,
}

pub struct NewTransaction {
    pub unique_string: String,
    pub sig: String,
    pub sender: String,
    pub moves: Vec<Move>,
}

impl Transaction {
    fn help_fmt(&self, f: &mut std::fmt::Formatter<'_>, sep: &str) -> std::fmt::Result {
        write!(f, "{}{}", self.serial, sep)?;
        write!(f, "transaction{}", sep)?;
        write!(f, "{}{}", self.unique_string, sep)?;
        write!(f, "{}{}", self.sig, sep)?;
        write!(f, "{}{}", self.sender, sep)?;
        let num_moves = self.moves.len();
        for (i, m) in self.moves.iter().enumerate() {
            write!(f, "{}", m)?;
            if i != num_moves - 1 {
                write!(f, "{}", sep)?;
            }
        }
        Ok(())
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.help_fmt(f, ":")
    }
}

impl Display for NewTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "transaction:")?;
        write!(f, "{}:", self.unique_string)?;
        write!(f, "{}:", self.sig)?;
        write!(f, "{}:", self.sender)?;
        let num_moves = self.moves.len();
        for (i, m) in self.moves.iter().enumerate() {
            write!(f, "{}", m)?;
            if i != num_moves - 1 {
                write!(f, ":")?;
            }
        }
        Ok(())
    }
}

impl FromStr for Transaction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split(':').collect::<Vec<&str>>();
        if split.len() < 5 {
            return Err("Transaction has less than five parts".to_string());
        }

        let serial = split.get(0).unwrap().to_string();
        let serial = match serial.parse::<u64>() {
            Ok(x) => x,
            Err(_) => return Err("Serial is not a number".to_string()),
        };

        // check second is transaction
        if split.get(1).unwrap() != &"transaction" {
            return Err("Second part is not transaction".to_string());
        }

        let unique_string = split.get(2).unwrap().to_string();
        // check it's actually base64
        let _ = base64::decode(&unique_string)
            .map_err(|_| "Unique string is not base64".to_string())?;

        // check it's at least 1 char
        if unique_string.is_empty() {
            return Err("Unique string is too short".to_string());
        }

        let sig = split.get(3).unwrap().to_string();
        // check it's actually base64
        let _ = base64::decode(&sig).map_err(|_| "Signature is not base64".to_string())?;

        // check it's right length
        if sig.len() != 88 {
            return Err("Signature has an invalid length".to_string());
        }

        let sender = split.get(4).unwrap().to_string();
        // check it's actually base64
        let _ = base64::decode(&sender)
            .map_err(|_| format!("Sender public key ({}) is not base64", sender))?;

        // check it's right length
        if sender.len() != 116 {
            return Err(format!("Sender public key ({}) is an invalid key", sender));
        }

        let moves = split.get(5..).unwrap().to_vec();
        let moves = moves
            .iter()
            .map(|x| x.parse::<Move>())
            .collect::<Result<Vec<Move>, String>>()?;
        Ok(Transaction {
            unique_string,
            serial,
            sig,
            sender,
            moves,
        })
    }
}

impl FromStr for NewTransaction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split(':').collect::<Vec<&str>>();
        if split.len() < 4 {
            return Err("Transaction has less than four parts".to_string());
        }

        // check first is transaction
        if split.get(0).unwrap() != &"transaction" {
            return Err("First part is not transaction".to_string());
        }

        let unique_string = split.get(1).unwrap().to_string();
        // check it's actually base64
        let _ = base64::decode(&unique_string)
            .map_err(|_| "Unique string is not base64".to_string())?;

        // check it's at least 1 char
        if unique_string.is_empty() {
            return Err("Unique string is too short".to_string());
        }

        let sig = split.get(2).unwrap().to_string();
        // check it's actually base64
        let _ = base64::decode(&sig).map_err(|_| "Signature is not base64".to_string())?;

        // check it's right length
        if sig.len() != 88 {
            return Err("Signature has an invalid length".to_string());
        }

        let sender = split.get(3).unwrap().to_string();
        // check it's actually base64
        let _ = base64::decode(&sender)
            .map_err(|_| format!("Sender public key ({}) is not base64", sender))?;

        // check it's right length
        if sender.len() != 116 {
            return Err(format!("Sender public key ({}) is an invalid key", sender));
        }

        let moves = split.get(4..).unwrap().to_vec();
        let moves = moves
            .iter()
            .map(|x| x.parse::<Move>())
            .collect::<Result<Vec<Move>, String>>()?;
        Ok(NewTransaction {
            unique_string,
            sig,
            sender,
            moves,
        })
    }
}

pub struct NewBlock {
    pub transactions: Vec<Transaction>,
    pub nonce: f64,
    pub miner_account: String,
}

pub struct Block {
    pub serial: u64,
    pub transactions: Vec<Transaction>,
    pub nonce: f64,
    pub miner_account: String,
}

impl NewBlock {
    pub fn genesis() -> Self {
        NewBlock {
            transactions: vec![],
            nonce: 1337.0,
            miner_account: format!(
                "{}{}",
                "AAAAB3NzaC1yc2EAAAADAQABAAAAQQDbXz4rfbrRrXYQJbwuC",
                "kIyIsccHRpxhxqxgKeneVF4eUXof6e2nLvdXkGA0Y6uBAQ6N7qKxasVTR/2s1N2OBWF"
            ),
        }
    }
}

impl FromStr for Block {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split(':').collect::<Vec<&str>>();
        if split.len() < 6 {
            return Err("Block has less than six parts".to_string());
        }

        let serial = split.get(0).unwrap().to_string();
        let serial = match serial.parse::<u64>() {
            Ok(x) => x,
            Err(_) => return Err("Serial is not a number".to_string()),
        };

        // check second is block
        if split.get(1).unwrap() != &"block" {
            return Err("Second part is not block".to_string());
        }

        let nonce = split.get(2).unwrap().to_string();
        let nonce = match nonce.parse::<f64>() {
            Ok(x) => x,
            Err(_) => return Err("Nonce is not a number".to_string()),
        };

        let miner_account = split.get(3).unwrap().to_string();
        // check it's actually base64
        let _ = base64::decode(&miner_account)
            .map_err(|_| "Miner account is not base64".to_string())?;

        // check it's right length
        if miner_account.len() != 116 {
            return Err("Miner account is an invalid key".to_string());
        }

        let transactions = split.get(4..).unwrap().to_vec();
        let transactions = transactions
            .iter()
            .map(|x| x.replace(';', ":").parse::<Transaction>())
            .collect::<Result<Vec<Transaction>, String>>()?;

        Ok(Block {
            serial,
            transactions,
            nonce,
            miner_account,
        })
    }
}

impl FromStr for NewBlock {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split(':').collect::<Vec<&str>>();
        if split.len() < 4 {
            return Err("Block has less than five parts".to_string());
        }

        // check second is block
        if split.get(0).unwrap() != &"block" {
            return Err("Second part is not block".to_string());
        }

        let nonce = split.get(1).unwrap().to_string();
        let nonce = match nonce.parse::<f64>() {
            Ok(x) => x,
            Err(_) => return Err("Nonce is not a number".to_string()),
        };

        let miner_account = split.get(2).unwrap().to_string();
        // check it's actually base64
        let _ = base64::decode(&miner_account)
            .map_err(|_| "Miner account is not base64".to_string())?;

        // check it's right length
        if miner_account.len() != 116 {
            return Err("Miner account is an invalid key".to_string());
        }

        let transactions = split.get(3..).unwrap().to_vec();
        let transactions = transactions
            .iter()
            .map(|x| x.replace(';', ":").parse::<Transaction>())
            .collect::<Result<Vec<Transaction>, String>>()?;

        Ok(NewBlock {
            transactions,
            nonce,
            miner_account,
        })
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", self.serial)?;
        write!(f, "block:")?;
        write!(f, "{}:", self.nonce)?;
        write!(f, "{}:", self.miner_account)?;
        let num_transactions = self.transactions.len();
        for (i, t) in self.transactions.iter().enumerate() {
            t.help_fmt(f, ";")?;
            if i != num_transactions - 1 {
                write!(f, ":")?;
            }
        }
        Ok(())
    }
}

impl Display for NewBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "block:")?;
        write!(f, "{}:", self.nonce)?;
        write!(f, "{}:", self.miner_account)?;
        let num_transactions = self.transactions.len();
        for (i, t) in self.transactions.iter().enumerate() {
            t.help_fmt(f, ";")?;
            if i != num_transactions - 1 {
                write!(f, ":")?;
            }
        }
        Ok(())
    }
}

pub enum NewMessage {
    NewTransaction(NewTransaction),
    NewBlock(NewBlock),
}

pub enum Message {
    Block(Block),
    Transaction(Transaction),
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Block(b) => write!(f, "{}", b),
            Message::Transaction(t) => write!(f, "{}", t),
        }
    }
}

impl Display for NewMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NewMessage::NewBlock(b) => write!(f, "{}", b),
            NewMessage::NewTransaction(t) => write!(f, "{}", t),
        }
    }
}

impl FromStr for Message {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split(':').collect::<Vec<&str>>();
        if split.len() < 2 {
            return Err("Message has less than two parts".to_string());
        }

        let which = split.get(1).unwrap();
        match *which {
            "block" => Ok(Message::Block(s.parse::<Block>()?)),
            "transaction" => Ok(Message::Transaction(s.parse::<Transaction>()?)),
            _ => Err("Message is not block or transaction".to_string()),
        }
    }
}

impl FromStr for NewMessage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split(':').collect::<Vec<&str>>();

        if split.len() < 2 {
            return Err("Message has less than two parts".to_string());
        }

        let which = split.get(0).unwrap();

        match *which {
            "block" => Ok(NewMessage::NewBlock(s.parse::<NewBlock>()?)),
            "transaction" => Ok(NewMessage::NewTransaction(s.parse::<NewTransaction>()?)),
            _ => Err("Message is not block or transaction".to_string()),
        }
    }
}

#[cfg(test)]
mod messages_tests {

    #[test]
    fn test_move_from_str() {
        let m = "a,1.0".parse::<super::Move>().unwrap();
        assert_eq!(m.from, "a");
        assert_eq!(m.amount, 1.0);
    }

    #[test]
    fn test_move_from_str_int() {
        let m = "a,1".parse::<super::Move>().unwrap();
        assert_eq!(m.from, "a");
        assert_eq!(m.amount, 1.0);
    }

    #[test]
    fn test_transaction_from_str() {
        let t = "0:transaction:Zm9v:1:2,2.0"
            .parse::<super::Transaction>()
            .unwrap();
        assert_eq!(t.serial, 0);
        assert_eq!(t.sig, "Zm9v");
        assert_eq!(t.moves.len(), 1);
        assert_eq!(t.sender, "1");
        assert_eq!(t.moves.get(0).unwrap().from, "2");
        assert_eq!(t.moves.get(0).unwrap().amount, 2.0);
    }

    #[test]
    fn test_new_transaction_from_str() {
        let t = "transaction:Zm9v:1:2,2.0"
            .parse::<super::NewTransaction>()
            .unwrap();
        assert_eq!(t.sig, "Zm9v");
        assert_eq!(t.moves.len(), 1);
        assert_eq!(t.sender, "1");
        assert_eq!(t.moves.get(0).unwrap().from, "2");
        assert_eq!(t.moves.get(0).unwrap().amount, 2.0);
    }
}
