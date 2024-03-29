use std::num;
use std::str;
use std::str::FromStr;
use std::string;

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::log_data::{LogData, StructuredData};

#[derive(Debug, Error)]
pub enum ParseErr {
    #[error("regular expression does not parse")]
    RegexDoesNotMatchErr,
    // #[error("bad severity in message")]
    // BadSeverityInPri,
    // #[error("bad facility in message")]
    // BadFacilityInPri,
    #[error(transparent)]
    TimestampParseError(#[from] chrono::ParseError),

    #[error("unexpected eof")]
    UnexpectedEndOfInput,
    #[error("too few digits in numeric field")]
    TooFewDigits,
    #[error("too many digits in numeric field")]
    TooManyDigits,
    #[error("invalid UTC offset")]
    InvalidUTCOffset,
    #[error("unicode error: {0}")]
    BaseUnicodeError(#[from] str::Utf8Error),
    #[error("unicode error: {0}")]
    UnicodeError(#[from] string::FromUtf8Error),
    #[error("unexpected input at character {0}")]
    ExpectedTokenErr(char),
    #[error("integer conversion error: {0}")]
    IntConversionErr(#[from] num::ParseIntError),
    #[error("missing field {0}")]
    MissingField(&'static str),
}

// We parse with this super-duper-dinky hand-coded recursive descent parser because we don't really
// have much other choice:
//
//  - Regexp is much slower (at least a factor of 4), and we still end up having to parse the
//    somewhat-irregular SD
//  - LALRPOP requires non-ambiguous tokenization
//  - Rust-PEG doesn't work on anything except nightly
//
// So here we are. The macros make it a bit better.
//
// General convention is that the parse state is represented by a string slice named "rest"; the
// macros will update that slice as they consume tokens.

macro_rules! maybe_expect_char {
    ($s:expr, $e: expr) => {
        match $s.chars().next() {
            Some($e) => Some(&$s[1..]),
            _ => None,
        }
    };
}

macro_rules! take_item {
    ($e:expr, $r:expr) => {{
        let (t, r) = $e?;
        $r = r;
        t
    }};
}

type ParseResult<T> = Result<T, ParseErr>;

macro_rules! take_char {
    ($e: expr, $c:expr) => {{
        $e = match $e.chars().next() {
            Some($c) => &$e[1..],
            Some(_) => {
                return Err(ParseErr::ExpectedTokenErr($c));
            }
            None => {
                return Err(ParseErr::UnexpectedEndOfInput);
            }
        }
    }};
}

fn take_while<F>(input: &str, f: F, max_chars: usize) -> (&str, Option<&str>)
where
    F: Fn(char) -> bool,
{
    for (idx, chr) in input.char_indices() {
        if !f(chr) {
            return (&input[..idx], Some(&input[idx..]));
        }
        if idx == max_chars {
            return (&input[..idx], Some(&input[idx..]));
        }
    }
    (input, None)
}

type ParsedParams = Vec<(String, String)>;

fn parse_data_key(input: &str) -> ParseResult<(String, &str)> {
    let (res, rest) = take_while(input, |c| c != ' ' && c != '=', 128);
    Ok((
        String::from(res),
        match rest {
            Some(s) => s,
            None => " ",
        },
    ))
}

fn parse_data_value(input: &str) -> ParseResult<(String, &str)> {
    let (res, rest) = take_while(input, |c| c != ' ', 128);
    Ok((
        String::from(res),
        match rest {
            Some(s) => &s[1..],
            None => " ",
        },
    ))
}

fn parse_data(msg: &str) -> ParseResult<(ParsedParams, &str)> {
    let mut data = Vec::new();

    let mut rest = msg;
    while !rest.is_empty() {
        let key = take_item!(parse_data_key(rest), rest);

        match maybe_expect_char!(rest, '=') {
            Some(rest2) => {
                rest = rest2;
                let value = take_item!(parse_data_value(rest), rest);
                data.push((key, String::from(&*value)));
            }
            None => {
                rest = &rest[1..];
                continue;
            }
        }
    }
    return Ok((data, rest));
}

fn parse_sd(structured_data_raw: &str) -> ParseResult<(StructuredData, &str)> {
    let mut sd = StructuredData::new_empty();
    let mut rest = structured_data_raw;
    while !rest.is_empty() {
        let params = take_item!(parse_data(rest), rest);
        for (sd_param_id, sd_param_value) in params {
            sd.insert_tuple(sd_param_id, sd_param_value);
        }
        if rest.starts_with(' ') {
            break;
        }
    }
    Ok((sd, rest))
}

fn parse_num(s: &str, min_digits: usize, max_digits: usize) -> ParseResult<(i32, &str)> {
    let (res, rest1) = take_while(s, |c| c >= '0' && c <= '9', max_digits);
    let rest = rest1.ok_or(ParseErr::UnexpectedEndOfInput)?;
    if res.len() < min_digits {
        Err(ParseErr::TooFewDigits)
    } else if res.len() > max_digits {
        Err(ParseErr::TooManyDigits)
    } else {
        Ok((
            i32::from_str(res).map_err(ParseErr::IntConversionErr)?,
            rest,
        ))
    }
}

fn parse_decimal(d: &str, min_digits: usize, max_digits: usize) -> ParseResult<(i32, &str)> {
    parse_num(d, min_digits, max_digits).map(|(val, s)| {
        let mut multiplicand = 1;
        let z = 10 - (d.len() - s.len());

        for _i in 1..(z) {
            multiplicand *= 10;
        }
        (val * multiplicand, s)
    })
}

fn parse_timestamp(input: &str) -> ParseResult<(DateTime<Utc>, &str)> {
    let (res, rest1) = take_while(input, |c| c != ' ', 128);
    let rest = rest1.ok_or(ParseErr::UnexpectedEndOfInput)?;
    if res.starts_with('-') {
        return Err(ParseErr::MissingField("timestamp"));
    }

    let dt = DateTime::parse_from_rfc3339(res)
        .map_err(ParseErr::TimestampParseError)?
        .with_timezone(&Utc);
    Ok((dt, rest))
}

fn parse_term(
    m: &str,
    min_length: usize,
    max_length: usize,
) -> ParseResult<(Option<String>, &str)> {
    if m.starts_with('-') && (m.len() <= 1 || m.as_bytes()[1] == 0x20) {
        return Ok((None, &m[1..]));
    }
    let byte_ary = m.as_bytes();
    for (idx, chr) in byte_ary.iter().enumerate() {
        if *chr < 33 || *chr > 126 {
            if idx < min_length {
                return Err(ParseErr::TooFewDigits);
            }
            let utf8_ary = str::from_utf8(&byte_ary[..idx]).map_err(ParseErr::BaseUnicodeError)?;
            return Ok((Some(String::from(utf8_ary)), &m[idx..]));
        }
        if idx >= max_length {
            let utf8_ary = str::from_utf8(&byte_ary[..idx]).map_err(ParseErr::BaseUnicodeError)?;
            return Ok((Some(String::from(utf8_ary)), &m[idx..]));
        }
    }
    Err(ParseErr::UnexpectedEndOfInput)
}

fn parse_appname(input: &str) -> ParseResult<(String, &str)> {
    parse_term(input, 1, 48).and_then(|(res, rest)| {
        res.ok_or(ParseErr::MissingField("appname"))
            .map(|r| (r, rest))
    })
}

fn parse_procid(input: &str) -> ParseResult<(String, &str)> {
    parse_term(input, 1, 128).and_then(|(res, rest)| {
        res.ok_or(ParseErr::MissingField("procid"))
            .map(|r| (r, rest))
    })
}

fn parse_line_s(m: &str) -> ParseResult<LogData> {
    let mut rest = m;
    let _len = take_item!(parse_num(rest, 1, 5), rest);
    take_char!(rest, ' ');
    take_char!(rest, '<');
    let _prival = take_item!(parse_num(rest, 1, 3), rest);
    take_char!(rest, '>');
    let _version = take_item!(parse_num(rest, 1, 2), rest);
    take_char!(rest, ' ');
    let timestamp = take_item!(parse_timestamp(rest), rest);
    take_char!(rest, ' ');
    let _hostname = take_item!(parse_term(rest, 1, 255), rest);
    take_char!(rest, ' ');
    let appname = take_item!(parse_appname(rest), rest);
    take_char!(rest, ' ');
    let procid = take_item!(parse_procid(rest), rest);
    take_char!(rest, ' ');
    let _msgid = take_item!(parse_term(rest, 1, 32), rest);
    take_char!(rest, ' ');
    rest = match maybe_expect_char!(rest, ' ') {
        Some(r) => r,
        None => rest,
    };
    // let data = take_item!(parse_sd(rest), rest);
    let msg = String::from(rest);

    Ok(LogData {
        timestamp,
        appname,
        procid,
        msg,
    })
}

fn parse_msg_s(m: &str) -> ParseResult<StructuredData> {
    let mut rest = m;
    let data = take_item!(parse_sd(rest), rest);
    Ok(data)
}

pub fn parse_line<S: AsRef<str>>(s: S) -> ParseResult<LogData> {
    parse_line_s(s.as_ref())
}

pub fn parse_msg<S: AsRef<str>>(s: S) -> ParseResult<StructuredData> {
    parse_msg_s(s.as_ref())
}

// #[cfg(test)]
// mod tests {
//     use std::collections::BTreeMap;
//     use std::mem;

//     use super::{parse_message, ParseErr};
//     // use crate::message;

//     use crate::facility::SyslogFacility;
//     use crate::severity::SyslogSeverity;

//     #[test]
//     fn test_simple() {
//         let msg = parse_message("<1>1 - - - - - -").expect("Should parse empty message");
//         assert!(msg.facility == SyslogFacility::LOG_KERN);
//         assert!(msg.severity == SyslogSeverity::SEV_ALERT);
//         assert!(msg.timestamp.is_none());
//         assert!(msg.hostname.is_none());
//         assert!(msg.appname.is_none());
//         assert!(msg.procid.is_none());
//         assert!(msg.msgid.is_none());
//         assert!(msg.sd.len() == 0);
//     }

//     #[test]
//     fn test_with_time_zulu() {
//         let msg = parse_message("<1>1 2015-01-01T00:00:00Z host - - - -")
//             .expect("Should parse empty message");
//         assert_eq!(msg.timestamp, Some(1420070400));
//     }

//     #[test]
//     fn test_with_time_offset() {
//         let msg = parse_message("<1>1 2015-01-01T00:00:00+00:00 - - - - -")
//             .expect("Should parse empty message");
//         assert_eq!(msg.timestamp, Some(1420070400));
//     }

//     #[test]
//     fn test_with_time_offset_nonzero() {
//         let msg = parse_message("<1>1 2015-01-01T00:00:00-10:00 - - - - -")
//             .expect("Should parse empty message");
//         assert_eq!(msg.timestamp, Some(1420106400));
//         // example from RFC 3339
//         let msg1 = parse_message("<1>1 2015-01-01T18:50:00-04:00 - - - - -")
//             .expect("Should parse empty message");
//         let msg2 = parse_message("<1>1 2015-01-01T22:50:00Z - - - - -")
//             .expect("Should parse empty message");
//         assert_eq!(msg1.timestamp, msg2.timestamp);
//         // example with fractional minutes
//         let msg1 = parse_message("<1>1 2019-01-20T00:46:39+05:45 - - - - -")
//             .expect("Should parse empty message");
//         let msg2 = parse_message("<1>1 2019-01-19T11:01:39-08:00 - - - - -")
//             .expect("Should parse empty message");
//         assert_eq!(msg1.timestamp, msg2.timestamp);
//     }

//     #[test]
//     fn test_complex() {
//         let msg = parse_message("<78>1 2016-01-15T00:04:01+00:00 host1 CROND 10391 - [meta sequenceId=\"29\"] some_message").expect("Should parse complex message");
//         assert_eq!(msg.facility, SyslogFacility::LOG_CRON);
//         assert_eq!(msg.severity, SyslogSeverity::SEV_INFO);
//         assert_eq!(msg.hostname, Some(String::from("host1")));
//         assert_eq!(msg.appname, Some(String::from("CROND")));
//         assert_eq!(msg.procid, Some(message::ProcId::PID(10391)));
//         assert_eq!(msg.msg, String::from("some_message"));
//         assert_eq!(msg.timestamp, Some(1452816241));
//         assert_eq!(msg.sd.len(), 1);
//         let v = msg
//             .sd
//             .find_tuple("meta", "sequenceId")
//             .expect("Should contain meta sequenceId");
//         assert_eq!(v, "29");
//     }

//     #[test]
//     fn test_sd_features() {
//         let msg = parse_message("<78>1 2016-01-15T00:04:01Z host1 CROND 10391 - [meta sequenceId=\"29\" sequenceBlah=\"foo\"][my key=\"value\"][meta bar=\"baz=\"] some_message").expect("Should parse complex message");
//         assert_eq!(msg.facility, SyslogFacility::LOG_CRON);
//         assert_eq!(msg.severity, SyslogSeverity::SEV_INFO);
//         assert_eq!(msg.hostname, Some(String::from("host1")));
//         assert_eq!(msg.appname, Some(String::from("CROND")));
//         assert_eq!(msg.procid, Some(message::ProcId::PID(10391)));
//         assert_eq!(msg.msg, String::from("some_message"));
//         assert_eq!(msg.timestamp, Some(1452816241));
//         assert_eq!(msg.sd.len(), 2);
//         assert_eq!(
//             msg.sd.find_sdid("meta").expect("should contain meta").len(),
//             3
//         );
//     }

//     #[test]
//     fn test_sd_with_escaped_quote() {
//         let msg_text = r#"<1>1 - - - - - [meta key="val\"ue"] message"#;
//         let msg = parse_message(msg_text).expect("should parse");
//         assert_eq!(
//             msg.sd
//                 .find_tuple("meta", "key")
//                 .expect("Should contain meta key"),
//             r#"val"ue"#
//         );
//     }

//     #[test]
//     fn test_other_message() {
//         let msg_text = r#"<190>1 2016-02-21T01:19:11+00:00 batch6sj - - - [meta sequenceId="21881798" x-group="37051387"][origin x-service="tracking"] metascutellar conversationalist nephralgic exogenetic graphy streng outtaken acouasm amateurism prenotice Lyonese bedull antigrammatical diosphenol gastriloquial bayoneteer sweetener naggy roughhouser dighter addend sulphacid uneffectless ferroprussiate reveal Mazdaist plaudite Australasian distributival wiseman rumness Seidel topazine shahdom sinsion mesmerically pinguedinous ophthalmotonometer scuppler wound eciliate expectedly carriwitchet dictatorialism bindweb pyelitic idic atule kokoon poultryproof rusticial seedlip nitrosate splenadenoma holobenthic uneternal Phocaean epigenic doubtlessly indirection torticollar robomb adoptedly outspeak wappenschawing talalgia Goop domitic savola unstrafed carded unmagnified mythologically orchester obliteration imperialine undisobeyed galvanoplastical cycloplegia quinquennia foremean umbonal marcgraviaceous happenstance theoretical necropoles wayworn Igbira pseudoangelic raising unfrounced lamasary centaurial Japanolatry microlepidoptera"#;
//         parse_message(msg_text).expect("should parse as text");
//     }

//     #[test]
//     fn test_bad_pri() {
//         let msg = parse_message("<4096>1 - - - - - -");
//         assert!(msg.is_err());
//     }

//     #[test]
//     fn test_bad_match() {
//         // we shouldn't be able to parse RFC3164 messages
//         let msg = parse_message("<134>Feb 18 20:53:31 haproxy[376]: I am a message");
//         assert!(msg.is_err());
//     }

//     #[test]
//     fn test_example_timestamps() {
//         // these are the example timestamps in the rfc

//         let msg = parse_message("<1>1 1985-04-12T23:20:50.52Z host - - - -")
//             .expect("Should parse empty message");
//         assert_eq!(msg.timestamp, Some(482196050));
//         assert_eq!(msg.timestamp_nanos, Some(520000000));

//         let msg = parse_message("<1>1 1985-04-12T19:20:50.52+04:00 host - - - -")
//             .expect("Should parse empty message");
//         assert_eq!(msg.timestamp, Some(482167250));
//         assert_eq!(msg.timestamp_nanos, Some(520000000));

//         let msg = parse_message("<1>1 1985-04-12T19:20:50+04:00 host - - - -")
//             .expect("Should parse empty message");
//         assert_eq!(msg.timestamp, Some(482167250));
//         assert_eq!(msg.timestamp_nanos, Some(0));

//         let msg = parse_message("<1>1 2003-08-24T05:14:15.000003+07:00 host - - - -")
//             .expect("Should parse empty message");
//         assert_eq!(msg.timestamp, Some(1061676855));
//         assert_eq!(msg.timestamp_nanos, Some(3000));

//         let msg = parse_message("<1>1 2003-08-24T05:14:15.000000003+07:00 host - - - -");
//         assert!(msg.is_err(), "expected parse fail");
//     }

//     #[test]
//     fn test_empty_sd_value() {
//         let msg = parse_message(r#"<29>1 2018-05-14T08:23:01.520Z leyal_test4 mgd 13894 UI_CHILD_EXITED [junos@2636.1.1.1.2.57 pid="14374" return-value="5" core-dump-status="" command="/usr/sbin/mustd"]"#).expect("must parse");
//         assert_eq!(msg.facility, SyslogFacility::LOG_DAEMON);
//         assert_eq!(msg.severity, SyslogSeverity::SEV_NOTICE);
//         assert_eq!(msg.hostname, Some(String::from("leyal_test4")));
//         assert_eq!(msg.appname, Some(String::from("mgd")));
//         assert_eq!(msg.procid, Some(message::ProcId::PID(13894)));
//         assert_eq!(msg.msg, String::from(""));
//         assert_eq!(msg.timestamp, Some(1526286181));
//         assert_eq!(msg.timestamp_nanos, Some(520000000));
//         assert_eq!(msg.sd.len(), 1);
//         let sd = msg
//             .sd
//             .find_sdid("junos@2636.1.1.1.2.57")
//             .expect("should contain root SD");
//         let expected = {
//             let mut expected = BTreeMap::new();
//             expected.insert("pid", "14374");
//             expected.insert("return-value", "5");
//             expected.insert("core-dump-status", "");
//             expected.insert("command", "/usr/sbin/mustd");
//             expected
//                 .into_iter()
//                 .map(|(k, v)| (k.to_string(), v.to_string()))
//                 .collect::<BTreeMap<_, _>>()
//         };
//         assert_eq!(sd, &expected);
//     }

//     #[test]
//     fn test_fields_start_with_dash() {
//         let msg = parse_message("<39>1 2018-05-15T20:56:58+00:00 -web1west -201805020050-bc5d6a47c3-master - - [meta sequenceId=\"28485532\"] 25450-uWSGI worker 6: getaddrinfo*.gaih_getanswer: got type \"DNAME\"").expect("should parse");
//         assert_eq!(msg.hostname, Some("-web1west".to_string()));
//         assert_eq!(
//             msg.appname,
//             Some("-201805020050-bc5d6a47c3-master".to_string())
//         );
//         assert_eq!(
//             msg.sd.find_tuple("meta", "sequenceId"),
//             Some(&"28485532".to_string())
//         );
//         assert_eq!(
//             msg.msg,
//             "25450-uWSGI worker 6: getaddrinfo*.gaih_getanswer: got type \"DNAME\"".to_string()
//         );
//     }

//     #[test]
//     fn test_truncated() {
//         let err =
//             parse_message("<39>1 2018-05-15T20:56:58+00:00 -web1west -").expect_err("should fail");
//         assert_eq!(
//             mem::discriminant(&err),
//             mem::discriminant(&ParseErr::UnexpectedEndOfInput)
//         );
//     }
// }
