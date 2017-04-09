use matching::{BaseMatch, MatchData};
use std::fmt;

/// Provides estimations of the time to crack a password given the number of
/// guesses required to crack it
#[derive(Default, Debug)]
pub struct CrackTimes {
    /// Online attack on a service with rate limiting 
    /// (100 per hour)
    online_throttling: f64,
    /// Offline attack on a service lacking or with compromised rate limiting 
    /// (10 per second)
    online_no_throttling: f64,
    /// Offline attack, assumes multiple attackers with a slow hash function
    /// (1e4 per second)
    offline_slow_hashing: f64,
    /// Offline attack with fast hash and multiple machines
    /// (1e10 per second)
    offline_fast_hashing: f64,
}

impl CrackTimes {
    pub fn new(guesses: u64) -> CrackTimes {
        let f_guess = guesses as f64;
        let ot = f_guess / (100.0f64 / 3600.0f64);
        let ont = f_guess / 10.0f64;
        let osh = f_guess / 1e4;
        let ofh = f_guess / 1e10;
        CrackTimes {
            online_throttling: ot,
            online_no_throttling: ont,
            offline_slow_hashing: osh,
            offline_fast_hashing: ofh,
        }
    }
}

fn seconds_to_string(seconds: f64) -> String {
    let minute = 60.0f64;
    let hour = minute * 60.0f64;
    let day = hour * 24.0f64;
    let month = day * 31.0f64;
    let year = month * 12.0f64;
    let century = year * 100.0f64;
    if seconds < 1.0f64 {
        String::from("less than a second")
    } else if seconds < minute {
        format!("{}s", seconds)
    } else if seconds < hour {
        format!("{} minute(s)", (seconds/minute).round())
    } else if seconds < day {
        format!("{} hour(s)", (seconds/hour).round())
    } else if seconds < month {
        format!("{} day(s)", (seconds/day).round())
    } else if seconds < year {
        format!("{} month(s)", (seconds/month).round())
    } else if seconds < century {
        format!("{} year(s)", (seconds/year).round())
    } else {
        String::from("centuries")
    }
}

impl fmt::Display for CrackTimes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "  Online throttled:\t{}", seconds_to_string(self.online_throttling))?;
        writeln!(f, "  Online unthrottled:\t{}", seconds_to_string(self.online_no_throttling))?;
        writeln!(f, "  Offline slow:\t\t{}", seconds_to_string(self.offline_slow_hashing))?;
        write!(f, "  Offline fast:\t\t{}", seconds_to_string(self.offline_fast_hashing))
    }
}

#[derive(PartialEq, Debug)]
pub enum PasswordScore {
    VeryWeak = 0,
    Weak = 1,
    Medium = 2,
    Strong = 3,
    VeryStrong = 4,
}

/// Feedback message for user.
/// Not necessarily required for users with strong passwords
#[derive(Debug)]
pub struct Feedback {
    /// Advice for creating stronger passwords  
    pub advice: String,
    /// Suggests how the password can be modified. e.g. add another word
    pub suggestions: String,
}

pub fn get_feedback(guesses: u64) -> PasswordScore {
    let guesses = guesses as f64;
    let delta = 5f64;
    match guesses {
        _ if guesses < 1e3 + delta => PasswordScore::VeryWeak,
        _ if guesses < 1e6 + delta => PasswordScore::Weak,
        _ if guesses < 1e8 + delta => PasswordScore::Medium,
        _ if guesses < 1e10 + delta => PasswordScore::Strong,
        _ => PasswordScore::VeryStrong,
    }
}

impl Default for Feedback {
    fn default() -> Feedback {
        Feedback {
            advice: String::new(),
            suggestions: String::from("Use a few words, avoid common phrases.\nNo need for \
                                       symbols, digits, or uppercase letters."),
        }
    }
}

/// zxcvbn-rs results for a given password.
/// TODO Implement a pretty print for struct to save having string fields for 
/// formatted data
#[derive(Default, Debug)]
pub struct PasswordResult {
    /// The password in question
    pub password: String,
    /// Estimated guesses to crack password
    pub guesses: u64,
    /// Order of magnitude of guesses
    pub guesses_log10: f64,
    /// Estimation of physical time to crack password
    pub crack_times: CrackTimes,
    /// Indicator of password quality
    pub score: Option<PasswordScore>,
    /// Feedback for the user based on password
    pub feedback: Option<Feedback>,
    /// Sequence of words in dictionary that results are based off
    pub sequence: Vec<BaseMatch>,
    /// Time for zxcvbn to calculate these results
    pub calculation_time: u32,
}

impl fmt::Display for PasswordResult {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "===============================================")?;
        writeln!(f, "Password:\t\t{}", self.password)?;
        writeln!(f, "Guesses raw:\t\t{}", self.guesses)?;
        writeln!(f, "Guesses log10:\t\t{}", self.guesses_log10)?;
        if let Some(ref score) = self.score {
            writeln!(f, "Score:\t\t\t{:#?}", score)?;
        }
        writeln!(f, "Guess times:")?;
        writeln!(f, "{}", self.crack_times)?;
        if let Some(ref feedback) = self.feedback {
            if !feedback.advice.is_empty() {
                writeln!(f, "{}", feedback.advice)?;
            }
            if !feedback.suggestions.is_empty() {
                writeln!(f, "{}", feedback.suggestions)?;
            }
        }
        writeln!(f, "===============================================")?;
        writeln!(f, "{:#?}", self.sequence)
    }
}


impl PasswordResult {
    pub fn get_feedback(&mut self) {
        self.score = Some(get_feedback(self.guesses));
        if self.sequence.is_empty() {
            self.feedback = Some(Feedback::default());
        }
        if let Some(ref s) = self.score {
            if s == &PasswordScore::Strong || s == &PasswordScore::VeryStrong {
                self.feedback = Some(Feedback{ 
                    suggestions: String::new(), 
                    ..Default::default()
                });
            } else {

                let longest_sequence = self.sequence.iter()
                                                    .max_by(|x, y| x.token.len()
                                                                          .cmp(&y.token.len()))
                                                    .unwrap();
                
                self.feedback = Some(self.get_match_feedback(longest_sequence, 
                                                             self.sequence.len() == 1));
            }
        }
        self.crack_times = CrackTimes::new(self.guesses);
    }

    fn get_match_feedback(&self, matched: &BaseMatch, only_match: bool) -> Feedback {
        match matched.data {
            MatchData::Dictionary{..} => self.get_dictionary_match_feedback(matched, only_match),
            MatchData::Spatial{ref turns, ..} => 
                Feedback { 
                    advice: 
                        if turns == &1 { 
                            String::from("Straight rows of keys are easier to guess") 
                        } else { 
                            String::from("Short keyboard patterns are easy to guess") 
                        },
                    suggestions: String::from("Use a longer keyboard pattern with more turns")
            },
            MatchData::Repeat{ref base_token, ..} => 
                Feedback {
                    advice: 
                        if base_token.chars().count() == 1 { 
                            String::from("Repeats like aaaa are easy to guess")
                        } else {
                            String::from("Repeats like abcabc are only slightly harder to guess than abc")
                        },
                    suggestions: String::from("Avoid repeated words and characters")
                },
            MatchData::Sequence{..} => 
                Feedback {
                    advice: String::from("Sequences like abc or 7654 are easy to guess"),
                    suggestions: String::from("Avoid sequences"),
                },
            MatchData::Regex{ref name} => 
                if name == &"recent year" { 
                    Feedback{
                        advice: String::from("Recent years are easy to guess"),
                        suggestions: String::from("Avoid recent years or years associated with you")
                    }
                } else {
                    Default::default()
                },
            MatchData::Date{..} => 
                Feedback {
                    advice: String::from("Dates are often easy to guess"),
                    suggestions: String::from("Avoid dates and years associated with you")
                },
            _ => Default::default(),
        }
    }

    fn get_dictionary_match_feedback(&self, 
                                     m: &BaseMatch, 
                                     only_match: bool) -> Feedback {

        if let MatchData::Dictionary{ref rank, ref dictionary_name, 
            ref reversed, ref l33t, ..} =m.data {
            
            let advice = if dictionary_name == &"Passwords" {
                if only_match && !l33t.is_some() && !*reversed {
                    if rank <= &10 {
                        "This is a top-10 common password"
                    } else if rank <= &100 {
                        "This is a top-100 common password"
                    } else {
                        "This is a very common password"
                    }
                } else if self.guesses_log10 <= 4.0f64 { 
                    "This is similar to a commonly used password"
                } else {
                    ""
                }
            } else if dictionary_name == &"Wikipedia" {
                if only_match {
                    "A word by itself is easy to guess"
                } else {
                    ""
                }
            } else if ["Male names", "Female names", "Surnames"].contains(&dictionary_name.as_ref()) {
                if only_match {
                    "Names and surnames by themselves are easy to guess"
                } else {
                    "Common names and surnames are easy to guess"
                }
            } else {
                ""
            };

            let suggestions = if *reversed {
                "Reversed words aren't mcuh harder to guess"
            } else if l33t.is_some() {
                "Predictable substitutions like '@' instead of 'a' don't help much"
            } else {
                ""
            };

            Feedback{
                advice: advice.to_string(),
                suggestions: suggestions.to_string()
            }
        } else {
            Default::default()
        }
    }
}
