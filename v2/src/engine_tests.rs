use super::*;
use serde_json::Value;
struct Recorded {
    trace: Vec<(usize, usize)>,
    at: usize,
}
impl Choice for Recorded {
    fn choose(&mut self, upper: usize) -> Result<usize, String> {
        let (bound, choice) = self.trace[self.at];
        self.at += 1;
        assert_eq!(upper, bound, "decision {}", self.at);
        Ok(choice)
    }
}
#[test]
fn actual_typescript_vectors_and_decision_ranges() {
    let vectors: Vec<Value> =
        serde_json::from_str(include_str!("../fixtures/oracle/engine.json")).unwrap();
    for vector in vectors {
        let mut rng = Recorded {
            trace: serde_json::from_value(vector["trace"].clone()).unwrap(),
            at: 0,
        };
        let input = &vector["input"];
        let result = match vector["kind"].as_str().unwrap() {
            "length" => parse_length(input.as_str().unwrap()).map(|n| serde_json::json!(n)),
            "random" => {
                let n = input["length"].as_f64().unwrap();
                if n.fract() != 0.0 {
                    Err("Password length must be between 5 and 64.".into())
                } else {
                    random_with(
                        n as usize,
                        input["options"]["numbers"].as_bool().unwrap(),
                        input["options"]["symbols"].as_bool().unwrap(),
                        &mut rng,
                    )
                    .map(|s| serde_json::json!(&*s))
                }
            }
            _ => memorable_with(input.as_str().unwrap(), &mut rng).map(|s| serde_json::json!(&*s)),
        };
        if let Some(error) = vector["error"].as_str() {
            assert_eq!(result.unwrap_err(), error, "input: {input}");
        } else {
            assert_eq!(result.unwrap(), vector["value"], "input: {input}");
        }
        assert_eq!(rng.at, rng.trace.len(), "input: {input}");
    }
}
#[test]
fn rejection_and_failure_are_bounded_and_closed() {
    let mut calls = 0;
    assert_eq!(
        bounded(10, |b| {
            calls += 1;
            b.fill(255);
            Ok(())
        })
        .unwrap_err(),
        ENTROPY_ERROR
    );
    assert_eq!(calls, 128);
    let mut calls = 0;
    assert_eq!(
        bounded(10, |b| {
            calls += 1;
            b.copy_from_slice(&if calls == 1 { u32::MAX } else { 19 }.to_le_bytes());
            Ok(())
        })
        .unwrap(),
        9
    );
    assert_eq!(calls, 2);
    assert!(bounded(64, |_| Err(())).is_err());
    assert!(bounded(0, |_| panic!("must not fill")).is_err());
    struct Fail;
    impl Choice for Fail {
        fn choose(&mut self, _: usize) -> Result<usize, String> {
            Err(ENTROPY_ERROR.into())
        }
    }
    assert!(random_with(20, true, true, &mut Fail).is_err());
    assert!(memorable_with("{word}", &mut Fail).is_err());
    struct Repeat;
    impl Choice for Repeat {
        fn choose(&mut self, _: usize) -> Result<usize, String> {
            Ok(0)
        }
    }
    assert_eq!(
        memorable_with("{word}-{word}", &mut Repeat).unwrap_err(),
        ENTROPY_ERROR
    );
}
#[test]
fn dictionary_is_complete_and_ordered() {
    let words: Vec<_> = WORDS.lines().collect();
    assert_eq!(words.len(), 2048);
    assert_eq!(words[0], "abandon");
    assert_eq!(words[2047], "zoo");
    assert_eq!(words.iter().collect::<HashSet<_>>().len(), 2048);
}
