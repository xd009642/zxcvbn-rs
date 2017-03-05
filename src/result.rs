///TODO Look at using chrono here!

/// Provides estimations of the time to crack a password given the number of
/// guesses required to crack it
#[derive(Default)]
struct CrackTimes {
    /// Online attack on a service with rate limiting 
    /// (100 per hour)
    online_throttling : f32,
    /// Offline attack on a service lacking or with compromised rate limiting 
    /// (10 per second)
    online_no_throttling : f32,
    /// Offline attack, assumes multiple attackers with a slow hash function
    /// (1e4 per second)
    offline_slow_hashing : f32,
    /// Offline attack with fast hash and multiple machines
    /// (1e10 per second)
    offline_fast_hashing : f32,
}

impl CrackTimes {
    fn new(guesses: u32) -> CrackTimes {
        let f_guess = guesses as f32;
        let ot = f_guess / (100.0f32 / 3600.0f32);
        let ont = f_guess / 10.0f32;
        let osh = f_guess / 1e4;
        let ofh = f_guess / 1e10;
        CrackTimes { online_throttling: ot, online_no_throttling: ont,
                     offline_slow_hashing: osh, offline_fast_hashing: ofh}
    }
}

enum PasswordScore {
    VeryWeak = 0,
    Weak = 1,
    Medium = 2,
    Strong = 3,
    VeryStrong = 4,
}

/// Feedback message for user.
/// Not necessarily required for users with strong passwords
struct Feedback {
    /// Advice for creating stronger passwords  
    advice: String,
    /// Describes what is wrong with the current password
    description: String,
    /// Suggests how the password can be modified. e.g. add another word
    suggestions: String,
}

impl Default for Feedback {
    fn default() -> Feedback {
        Feedback { advice: String::new(), description: String::new(), 
                   suggestions: String::from("Use a few words, \
                   avoid common phrases.\n\
                   No need for symbols, digits, or uppercase letters.")}
    }
}

/// zxcvbn-rs results for a given password.
/// TODO Implement a pretty print for struct to save having string fields for 
/// formatted data
#[derive(Default)]
struct Result {
    /// Estimated guesses to crack password
    guesses: u32,
    /// Order of magnitude of guesses
    guesses_log10: f64,
    /// Estimation of physical time to crack password
    crack_times: CrackTimes,
    /// Indicator of password quality
    score: Option<PasswordScore>,
    /// Feedback for the user based on password
    feedback: Option<Feedback>,
    /// Sequence of words in dictionary that results are based off
    sequence: Vec<String>,
    /// Time for zxcvbn to calculate these results
    calculation_time: u32,
}


fn get_match_feedback(matched: &String, only_match:bool) -> String {

    String::new()
}

impl Result {
    fn get_feedback(&mut self, guesses: u32, matches: &Vec<String>) {
        if matches.is_empty() {
            self.feedback = Some(Feedback::default());
        }
        if let Some(ref s) = self.score {
            let longest_sequence = matches.iter()
                .max_by(|x, y| x.len().cmp(&y.len()))
                .unwrap();
            let feedback = get_match_feedback(longest_sequence, 
                                              matches.len()==1);
        }
    }

}
