use std::str;
use {transform_u64_to_array_of_u8, transform_array_of_u8_to_u64};

#[derive(Debug, PartialEq)]
pub struct Mail {
    pub account: Option<String>,
    pub recipients: Vec<String>,
    pub body: Vec<u8>,
}

impl Mail {
    const MAGIC_NUMBER: &'static str = "RUSMTP";
    const VERSION_MAJOR: u8 = 1;
    const VERSION_MINOR: u8 = 0;

    pub fn serialize(&self) -> Vec<u8> {
        let mut sink = Vec::new();
        // Write the magic number
        sink.extend_from_slice(Mail::MAGIC_NUMBER.as_bytes());

        // Write the version
        sink.push(Mail::VERSION_MAJOR);
        sink.push(Mail::VERSION_MINOR);

        // Write the length of the bytes in the account name and
        // the account itself if it exists, or write 0
        match &self.account {
            None          => sink.push(0),
            Some(account) => {
                let account_bytes = account.as_bytes();
                sink.push(account_bytes.len() as u8);
                sink.extend_from_slice(account_bytes);
            },
        };

        // Write the length of the bytes in the recipients and
        // the actual bytes of the recipients
        for recipient in &self.recipients {
            let recipient_bytes = recipient.as_bytes();
            sink.push(recipient_bytes.len() as u8);
            sink.extend_from_slice(recipient_bytes);
        }

        // Write an indicator that the recipients are finished
        sink.push(0);
        sink.extend_from_slice(&transform_u64_to_array_of_u8(self.body.len() as u64));

        // Write the body of the email
        sink.extend(&self.body);
        sink
    }

    fn sanity_check(bytes: &[u8]) -> bool {
        // check the length of the bytes
        // magic number
        let mut expected_length = Mail::MAGIC_NUMBER.len();
        // major version
        expected_length += 1;
        // minor version
        expected_length += 1;
        // account name length
        expected_length += 1;
        // add account length
        match bytes.get(expected_length - 1) {
            Some(&value) =>
                expected_length += value as usize,
            None         =>
                return false,
        };
        // add combined recipients length
        loop {
            // recipient length
            expected_length += 1;
            match bytes.get(expected_length - 1) {
                Some(0)      => break,
                Some(&value) =>
                    expected_length += value as usize,
                None         =>
                    return false,
            };
        }

        let bound_start = expected_length - 1;
        let bound_end = expected_length + 7;
        let mut expected_length = expected_length as u64;
        match bytes.get(bound_start..bound_end) {
            None        => return false,
            Some(slice) => {
                let body_length = transform_array_of_u8_to_u64(slice);
                expected_length += body_length + 8;
            },

        }
        if (bytes.len() as u64) < expected_length {
            return false;
        }

        true
    }

    pub fn deserialize(bytes: &mut Vec<u8>) -> Result<Self, String> {
        if ! Mail::sanity_check(bytes) {
            return Err("Message unexpectedly truncated".to_string());
        }

        // Read and check magic number
        let magic_len = Mail::MAGIC_NUMBER.len();
        let possible_magic: Vec<u8> = bytes.drain(0..magic_len).collect();
        if possible_magic.as_slice() != Mail::MAGIC_NUMBER.as_bytes() {
            return Err("Bad magic number for message".to_string());
        }

        // Read and check major version
        if bytes.remove(0) != Mail::VERSION_MAJOR {
            return Err("Bad major version number for message".to_string());
        };

        // Read and check minor version
        if bytes.remove(0) != Mail::VERSION_MINOR {
            return Err("Bad minor version number for message".to_string());
        };

        // Read account
        let next = bytes.remove(0);
        let account = match next {
            0    => None,
            size => {
                let acc: Vec<u8> = bytes.drain(0..size as usize).collect();
                match str::from_utf8(acc.as_slice()) {
                    Ok(acc) => Some(acc.to_string()),
                    Err(_)  => return Err("Invalid account name".to_string())
                }
            }
        };

        // Read recipients
        let mut recipients = Vec::new();
        loop {
            let next = bytes.remove(0);
            if next == 0 { break; }
            let recipient: Vec<u8> = bytes.drain(0..next as usize).collect();
            match str::from_utf8(recipient.as_slice()) {
                Ok(recipient) => {
                    recipients.push(recipient.to_string());
                },
                Err(_)        =>
                    return Err("Invalid recipient".to_string())
            }

        }

        // Read the size of the body
        let _ = bytes.drain(0..8);

        // Read the body of the message
        let body = bytes.to_owned();

        Ok(Mail {
            account,
            recipients,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_account_email() {
        let expected = Mail {
            account: None,
            recipients: vec!["f@s.s".to_string(), "s@t.f".to_string()],
            body: b"valuable email".to_vec(),
        };

        let mut serialized = expected.serialize();
        let actual = Mail::deserialize(&mut serialized);
        assert_eq!(Ok(expected), actual);
    }

    #[test]
    fn test_not_empty_account_email() {
        let expected = Mail {
            account: Some("first".to_string()),
            recipients: vec!["f@s.s".to_string(), "s@t.f".to_string()],
            body: b"valuable email".to_vec(),
        };

        let mut serialized = expected.serialize();
        let actual = Mail::deserialize(&mut serialized);
        assert_eq!(Ok(expected), actual);
    }
}
