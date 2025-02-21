use rand::Rng;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

mod mod_exp;
use mod_exp::mod_exp;

const P: u64 = 23;
const G: u64 = 5;

#[derive(Debug)]
struct Participant {
    name: String,
    private_key: u64,
    shared_secret: Option<u64>,
    answer_sender: Sender<CommandAnswer>,
    command_receiver: Receiver<Command>,
}

impl Participant {
    pub fn compute_exp(&self, intermediate_secret: u64) -> u64 {
        mod_exp(intermediate_secret, self.private_key, P)
    }

    pub fn run(mut self) {
        println!(
            "Running participant {} with private key {}",
            self.name.as_str(),
            self.private_key
        );

        for msg in &self.command_receiver {
            match msg {
                Command::ReceiveIntermediate { key } => {
                    let updated_intermediate = self.compute_exp(key);
                    let answer = CommandAnswer::UpdatedIntermediate {
                        key: updated_intermediate,
                    };

                    println!(
                        "Participant \"{}\" answer with update intermediate key {}",
                        self.name.as_str(),
                        updated_intermediate,
                    );

                    self.answer_sender
                        .send(answer)
                        .expect("Failed to send the answer back to coordinator");
                }
                Command::ReceiveFinal { key } => {
                    self.shared_secret = Some(self.compute_exp(key));

                    println!(
                        "Participant \"{}\" determined shared secret key to be {}",
                        self.name.as_str(),
                        self.shared_secret.unwrap(),
                    );
                }
            }
        }
    }
}

#[derive(Debug)]
enum Command {
    ReceiveIntermediate { key: u64 },
    ReceiveFinal { key: u64 },
}

#[derive(Debug)]
enum CommandAnswer {
    UpdatedIntermediate { key: u64 },
}

#[derive(Debug)]
struct ParticipantInfo {
    name: String,
    sender: Sender<Command>,
}

fn setup_participants() -> Vec<ParticipantInfo> {
    let mut participants = Vec::new();

    let mut rng = rand::rng();

    let (coordinator_sender, coordinator_receive) = channel::<CommandAnswer>();

    for name in ["A", "B", "C", "D", "E", "F", "G", "H"].iter() {
        let (tx, rx) = channel::<Command>();

        let private_key = rng.random_range(2..P);
        let participant = Participant {
            name: name.to_string(),
            private_key,
            shared_secret: Option::None,
            answer_sender: coordinator_sender.clone(),
            command_receiver: rx,
        };

        thread::spawn(move || participant.run());

        participants.push(ParticipantInfo {
            name: name.to_string(),
            sender: tx,
        });
    }

    loop {
        let mid = participants.len() / 2;
        let first_half = &participants[..mid];
        let second_half = &participants[mid..];

        print!("First half names: ");
        first_half.iter().for_each(|participant| {
            print!("{} ", participant.name.as_str());
        });
        print!("\n");

        print!("Second half names: ");
        second_half.iter().for_each(|participant| {
            print!("{} ", participant.name.as_str());
        });
        print!("\n");

        for participant in first_half {

        }

        break;
    }

    for participant in &participants {
        let cmd = Command::ReceiveIntermediate { key: G };
        participant.sender.send(cmd).unwrap();

        let answer = coordinator_receive.recv().unwrap();
        println!("Received first answer: {:?}", &answer);

        let key = match answer {
            CommandAnswer::UpdatedIntermediate { key } => key,
        };
        let cmd = Command::ReceiveFinal { key };
        participant.sender.send(cmd).expect(
            format!(
                "Failed to send command message to participant {}",
                participant.name.as_str()
            )
            .as_str(),
        );
    }

    participants
}

fn main() {
    mod_exp(10, 3, 4);

    setup_participants();

    println!("Hello, world!");
}
