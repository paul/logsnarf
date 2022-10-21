use std::collections::BTreeMap;
use std::str;
use std::string;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected eof")]
    UnexpectedEndOfInput,
    #[error("missing field {0}")]
    MissingField(&'static str),
    #[error("unicode error: {0}")]
    BaseUnicodeError(#[from] str::Utf8Error),
    #[error("unicode error: {0}")]
    UnicodeError(#[from] string::FromUtf8Error),
}

type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct LogData {
    pub timestamp_str: String,
    pub hostname: String,
    pub appname: String,
    pub procid: String,
    pub msgid: Option<String>,
    pub msg: String,
}

pub type KVPairs = BTreeMap<String, String>;

/// Quickly parses a syslog-ish line into LogData
pub fn parse_line(m: &str) -> ParseResult<Option<LogData>> {
    let mut rest = m;

    rest = skip_to_after('>', rest)?;
    rest = skip_to_after(' ', rest)?;
    let (timestamp_str, rest) = parse_term(rest)?;
    let (hostname, rest) = parse_term(rest)?;
    let (appname, rest) = parse_term(rest)?;
    let (procid, rest) = parse_term(rest)?;
    let (msgid, rest) = parse_term(rest)?;
    let msg = String::from(rest);

    if let (Some(timestamp_str), Some(hostname), Some(appname), Some(procid)) =
        (timestamp_str, hostname, appname, procid)
    {
        Ok(Some(LogData {
            timestamp_str,
            hostname,
            appname,
            procid,
            msgid,
            msg,
        }))
    } else {
        Ok(None)
    }
}

// Parses a syslog "message" into key-value pairs, honoring quotes
pub fn parse_msg(msg: &str) -> ParseResult<KVPairs> {
    let mut rest = msg;
    let mut pairs = KVPairs::new();

    loop {
        let (key, rest2) = parse_key(rest)?;
        rest = rest2;
        if key.is_none() {
            continue;
        };
        let (val, rest) = parse_value(rest)?;
        pairs.insert(key.unwrap(), val);
        if rest.is_empty() {
            break;
        }
    }

    Ok(pairs)
}

fn skip_to_after(c: char, m: &str) -> ParseResult<&str> {
    for (idx, chr) in m.char_indices() {
        if chr == c {
            return Ok(&m[(idx + 1)..]);
        }
    }
    Err(ParseError::UnexpectedEndOfInput)
}

fn parse_term(m: &str) -> ParseResult<(Option<String>, &str)> {
    // Blank field
    if m.starts_with('-') && (m.len() <= 1 || m.as_bytes()[1] == 0x20) {
        return Ok((None, &m[2..]));
    }

    // Read until we get a Space or some unprintable ascii
    let byte_ary = m.as_bytes();
    for (idx, chr) in byte_ary.iter().enumerate() {
        if *chr < 33 || *chr > 126 {
            let utf8_ary =
                str::from_utf8(&byte_ary[..idx]).map_err(ParseError::BaseUnicodeError)?;
            return Ok((Some(String::from(utf8_ary)), &m[(idx + 1)..]));
        }
    }
    Err(ParseError::UnexpectedEndOfInput)
}

fn parse_key(input: &str) -> ParseResult<(Option<String>, &str)> {
    for (idx, chr) in input.char_indices() {
        if chr == '=' {
            let key = String::from(&input[..idx]);
            return Ok((Some(key), &input[(idx + 1)..]));
        } else if chr == ' ' {
            return Ok((None, &input[(idx + 1)..]));
        }
    }
    Err(ParseError::UnexpectedEndOfInput)
}

fn parse_value(input: &str) -> ParseResult<(String, &str)> {
    let mut quoted = false;
    let mut chars = input.char_indices().peekable();
    while let Some((idx, chr)) = chars.next() {
        if quoted {
            if chr == '"' {
                return Ok((String::from(&input[1..idx]), &input[(idx + 1)..]));
            }
        } else {
            if chr == '"' {
                quoted = true;
                continue;
            }
            if chr == ' ' {
                return Ok((String::from(&input[..idx]), &input[(idx + 1)..]));
            }
            if chars.peek().is_none() {
                return Ok((String::from(input), &""));
            }
        }
    }
    Err(ParseError::UnexpectedEndOfInput)
}

#[cfg(test)]
mod tests {
    use super::{parse_line, parse_msg};

    use std::fs::File;
    use std::io::{self, BufRead};

    #[test]
    fn test_it_parses() {
        let line = r#"302 <158>1 2019-11-25T18:28:00.089034+00:00 host heroku router - at=info method=GET"#;

        let r = parse_line(line)
            .expect("Should parse a line")
            .expect("Should have data");
        assert_eq!(
            r.timestamp_str,
            "2019-11-25T18:28:00.089034+00:00".to_string()
        );
        assert_eq!(r.hostname, "host".to_string());
        assert_eq!(r.appname, "heroku".to_string());
        assert_eq!(r.procid, "router".to_string());
        assert_eq!(r.msgid, None);
        assert_eq!(r.msg, "at=info method=GET".to_string());
    }

    #[test]
    fn test_it_parses_samples() {
        let names = [
            "dyno_load.log",
            "dyno_mem.log",
            "postgres.log",
            "redis.log",
            "router.log",
        ];

        for name in names {
            let file = File::open(format!("samples/{}", name))
                .expect(&format!("Cannot find file: {}", name));
            let lines = io::BufReader::new(file).lines();
            for line in lines {
                if let Ok(l) = line {
                    assert!(parse_line(&l).is_ok());
                }
            }
        }
    }

    #[test]
    fn test_it_parses_a_huge_file() {
        let file =
            File::open("samples/staging.log").expect(&format!("Cannot find file: staging.log"));
        let lines = io::BufReader::new(file).lines();

        for line in lines {
            if let Ok(l) = line {
                assert!(parse_line(&l).is_ok());
            }
        }
    }

    #[test]
    fn test_it_parses_missing_timestamp() {
        let line = r#"302 <158>1 - host heroku router - at=info method=GET"#;

        let r = parse_line(line).expect("Should parse the line");
        assert!(r.is_none())
    }

    #[test]
    fn test_it_parses_kv_pairs() {
        let msg = r#"at=info method=GET"#;
        let r = parse_msg(msg).expect("Should parse the message");

        assert_eq!(r.get(&"at".to_string()), Some(&"info".to_string()));
        assert_eq!(r.get(&"method".to_string()), Some(&"GET".to_string()));
    }

    #[test]
    fn test_it_parses_quoted_pair_values() {
        let msg = r#"path="/admin/sidekiq_queue_stats" fwd="52.90.232.237,70.132.60.79""#;
        let r = parse_msg(msg).expect("Should parse the message");

        assert_eq!(
            r.get(&"path".to_string()),
            Some(&"/admin/sidekiq_queue_stats".to_string())
        );
        assert_eq!(
            r.get(&"fwd".to_string()),
            Some(&"52.90.232.237,70.132.60.79".to_string())
        );
    }
}
