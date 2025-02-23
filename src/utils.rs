use rand::Rng;

pub fn generate_random_message() -> String {
    let rng = rand::rng();
    rng.sample_iter(&rand::distr::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

