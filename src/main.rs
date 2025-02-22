use rand::Rng;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

mod mod_exp;
use mod_exp::mod_exp;

const PARTICIPANTS_NAMES: [&str; 8] = ["A", "B", "C", "D", "E", "F", "G", "H"];

const P: u64 = 3083;
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
                Command::Exponentiate { key } => {
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
                Command::ReceiveIntermediate { key } => {}
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
    Exponentiate { key: u64 },
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

fn print_names(prefix: &str, participants: &[ParticipantInfo]) {
    print!("{} ", prefix);
    participants.iter().for_each(|participant| {
        print!("{} ", participant.name.as_str());
    });
    print!("\n");
}

fn split_into_groups(
    participants: &[ParticipantInfo],
    receiver: &Receiver<CommandAnswer>,
    intermediate_key: u64,
) {
    if participants.len() < 2 {
        panic!("Participant length is too small to be split into groups!");
    }

    let mid = participants.len() / 2;

    let first_half = &participants[..mid];
    let second_half = &participants[mid..];

    print_names("First half names:", first_half);
    print_names("Second half names:", second_half);

    let mut first_intermediate_key = intermediate_key;
    for participant in first_half {
        let cmd = Command::Exponentiate {
            key: first_intermediate_key,
        };
        participant.sender.send(cmd).unwrap();

        let answer = receiver.recv().unwrap();
        first_intermediate_key = match answer {
            CommandAnswer::UpdatedIntermediate { key } => key,
        };
    }

    let mut second_intermediate_key = intermediate_key;
    for participant in second_half {
        let cmd = Command::Exponentiate {
            key: second_intermediate_key,
        };
        participant.sender.send(cmd).unwrap();

        let answer = receiver.recv().unwrap();
        second_intermediate_key = match answer {
            CommandAnswer::UpdatedIntermediate { key } => key,
        };
    }

    if first_half.len() == 1 {
        let cmd = Command::ReceiveFinal {
            key: second_intermediate_key,
        };
        first_half[0].sender.send(cmd).unwrap();
    } else {
        split_into_groups(first_half, receiver, second_intermediate_key);
    }

    if second_half.len() == 1 {
        let cmd = Command::ReceiveFinal {
            key: first_intermediate_key,
        };
        second_half[0].sender.send(cmd).unwrap();
    } else {
        split_into_groups(second_half, receiver, first_intermediate_key);
    }
}

fn setup_participants() -> Vec<ParticipantInfo> {
    let mut participants = Vec::new();

    let mut rng = rand::rng();

    let (coordinator_sender, coordinator_receive) = channel::<CommandAnswer>();

    for name in PARTICIPANTS_NAMES.iter() {
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

    split_into_groups(&participants, &coordinator_receive, G);

    println!("Finished!!!");

    //for participant in &participants {
    //    let cmd = Command::Exponentiate { key: G };
    //    participant.sender.send(cmd).unwrap();
    //
    //    let answer = coordinator_receive.recv().unwrap();
    //    println!("Received first answer: {:?}", &answer);
    //
    //    let key = match answer {
    //        CommandAnswer::UpdatedIntermediate { key } => key,
    //    };
    //    let cmd = Command::ReceiveFinal { key };
    //    participant.sender.send(cmd).expect(
    //        format!(
    //            "Failed to send command message to participant {}",
    //            participant.name.as_str()
    //        )
    //        .as_str(),
    //    );
    //}

    participants
}

fn main() {
    mod_exp(10, 3, 4);

    setup_participants();

    println!("Hello, world!");
}
