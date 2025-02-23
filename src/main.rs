use std::fmt;
use rand::Rng;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

mod crypto;
mod mod_exp;
mod utils;

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

#[derive(Debug)]
enum Command {
    Exponentiate { key: u64 },
    FinalExponentiate { key: u64 },
    GenerateMessage,
    ReceiveMessage { from: String, msg: Vec<u8> },
}

#[derive(Debug)]
enum CommandAnswer {
    UpdatedIntermediate { key: u64 },
    UpdatedFinal,
    GeneratedMessage { msg: Vec<u8> },
    ReceivedMessage,
}

impl Participant {
    fn compute_exp(&self, intermediate_secret: u64) -> u64 {
        mod_exp::mod_exp(intermediate_secret, self.private_key, P)
    }

    pub fn run(mut self) {
        for msg in &self.command_receiver {
            match msg {
                Command::Exponentiate { key } => {
                    let updated_intermediate = self.compute_exp(key);
                    let answer = CommandAnswer::UpdatedIntermediate {
                        key: updated_intermediate,
                    };

                    println!(
                        "Participant \"{}\" answer with update intermediate key {} after receiving intermediate key {}",
                        self.name.as_str(),
                        updated_intermediate,
                        key
                    );

                    self.answer_sender
                        .send(answer)
                        .expect("Failed to send the answer back to coordinator");
                }
                Command::FinalExponentiate { key } => {
                    self.shared_secret = Some(self.compute_exp(key));

                    println!(
                        "Participant \"{}\" determined shared secret key to be {} after receiving intermediate key {}",
                        self.name.as_str(),
                        self.shared_secret.unwrap(),
                        key
                    );

                    let answer = CommandAnswer::UpdatedFinal;
                    self.answer_sender
                        .send(answer)
                        .expect("Failed to send final answer back to coordinator");
                }
                Command::GenerateMessage => {
                    let msg = utils::generate_random_message();

                    println!(
                        "Participant \"{}\" generated message {:?}",
                        self.name.as_str(),
                        msg.as_str()
                    );

                    let encrypted_msg =
                        crypto::encode_message(self.shared_secret.unwrap(), msg.as_str());

                    println!(
                        "Participant \"{}\" encrypted message {:?}",
                        self.name.as_str(),
                        encrypted_msg.as_slice()
                    );

                    let answer = CommandAnswer::GeneratedMessage { msg: encrypted_msg };
                    self.answer_sender
                        .send(answer)
                        .expect("Failed to send generated message");
                }

                Command::ReceiveMessage { from, msg } => {
                    println!(
                        "Participant \"{}\" received encrypted message {:?} from {}",
                        self.name.as_str(),
                        msg.as_slice(),
                        from.as_str()
                    );

                    let decrypted_msg =
                        crypto::decode_message(self.shared_secret.unwrap(), msg.as_slice());

                    println!(
                        "Participant \"{}\" decoded message {} from {}",
                        self.name.as_str(),
                        decrypted_msg.as_str(),
                        from.as_str()
                    );

                    let answer = CommandAnswer::ReceivedMessage;
                    self.answer_sender
                        .send(answer)
                        .expect("Failed to send received message acknowlegment");
                }
            }
        }
    }
}

struct ParticipantInfo {
    name: String,
    sender: Sender<Command>,
}

impl fmt::Debug for ParticipantInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name.as_str())
    }
}

struct Coordinator {
    participants: Vec<ParticipantInfo>,
    receiver: Receiver<CommandAnswer>,
}

impl Coordinator {
    pub fn new() -> Self {
        let mut participants = Vec::new();
        let mut rng = rand::rng();
        let (sender, receiver) = channel::<CommandAnswer>();

        for name in PARTICIPANTS_NAMES.iter() {
            let (tx, rx) = channel::<Command>();

            let private_key = rng.random_range(2..P);
            let participant = Participant {
                name: name.to_string(),
                private_key,
                shared_secret: Option::None,
                answer_sender: sender.clone(),
                command_receiver: rx,
            };

            thread::spawn(move || participant.run());

            participants.push(ParticipantInfo {
                name: name.to_string(),
                sender: tx,
            });
        }

        Coordinator {
            participants,
            receiver,
        }
    }

    fn run(&self) {
        self.recursive_calculate_secret_shared_key(&self.participants, G);
        self.generate_and_send_message();
    }

    fn recursive_calculate_secret_shared_key(&self, participants: &[ParticipantInfo], intermediate_key: u64) {
        if participants.len() < 2 {
            panic!("Participant length is too small to be split into groups!");
        }

        let mid = participants.len() / 2;
        let (first_half, second_half) = participants.split_at(mid);

        println!("First half names: {:?}", first_half);
        println!("Second half names: {:?}", second_half);

        let first_accumulated_key = self.accumulate_intermediate_key(first_half, intermediate_key);
        let second_accumulated_key =
            self.accumulate_intermediate_key(second_half, intermediate_key);

        if first_half.len() == 1 {
            self.exponentiate_final_secret_shared_key(&first_half[0], second_accumulated_key);
        } else {
            self.recursive_calculate_secret_shared_key(first_half, second_accumulated_key);
        }

        if second_half.len() == 1 {
            self.exponentiate_final_secret_shared_key(&second_half[0], first_accumulated_key);
        } else {
            self.recursive_calculate_secret_shared_key(second_half, first_accumulated_key);
        }
    }

    fn accumulate_intermediate_key(
        &self,
        participants: &[ParticipantInfo],
        intermediate_key: u64,
    ) -> u64 {
        participants
            .iter()
            .fold(intermediate_key, |acc_key, participant| {
                let cmd = Command::Exponentiate { key: acc_key };
                participant.sender.send(cmd).unwrap();

                let answer = self.receiver.recv().unwrap();
                match answer {
                    CommandAnswer::UpdatedIntermediate { key } => key,
                    _ => {
                        panic!("Should have received updated intermediate key!")
                    }
                }
            })
    }

    fn exponentiate_final_secret_shared_key(
        &self,
        participant: &ParticipantInfo,
        intermediate_key: u64,
    ) {
        let cmd = Command::FinalExponentiate {
            key: intermediate_key,
        };
        participant.sender.send(cmd).unwrap();

        let answer = self.receiver.recv().unwrap();
        match answer {
            CommandAnswer::UpdatedFinal => (),
            _ => panic!("Should have received final!"),
        }
    }

    fn generate_and_send_message(&self) {
        let mut rng = rand::rng();

        let rand_participant_index = rng.random_range(0..self.participants.len());
        let rand_participant = &self.participants[rand_participant_index];

        let cmd = Command::GenerateMessage;
        rand_participant.sender.send(cmd).unwrap();

        let answer = self.receiver.recv().unwrap();
        let generated_encrypted_message = match answer {
            CommandAnswer::GeneratedMessage { msg } => msg,
            _ => panic!("Expected generated message answer"),
        };

        for i in 0..self.participants.len() {
            if i != rand_participant_index {
                let cmd = Command::ReceiveMessage {
                    from: rand_participant.name.clone(),
                    msg: generated_encrypted_message.clone(),
                };
                self.participants[i].sender.send(cmd).unwrap();

                let answer = self.receiver.recv().unwrap();
                match answer {
                    CommandAnswer::ReceivedMessage => (),
                    _ => panic!("Expected received message answer"),
                }
            }
        }

        println!("Finished sending out messages");
    }
}

fn main() {
    let coordinator = Coordinator::new();
    coordinator.run();
}
