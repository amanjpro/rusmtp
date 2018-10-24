use std::str;

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

        // Write the body of the email
        sink.extend(&self.body);
        sink
    }

    pub fn deserialize(bytes: &mut Vec<u8>) -> Self {
        // Read and check magic number
        let magic_len = Mail::MAGIC_NUMBER.len();
        let possible_magic: Vec<u8> = bytes.drain(0..magic_len).collect();
        if possible_magic.as_slice() != Mail::MAGIC_NUMBER.as_bytes() {
            panic!("Bad magic number for message");
        }

        // Read and check major version
        if bytes.remove(0) != Mail::VERSION_MAJOR {
            panic!("Bad major version number for message");
        };

        // Read and check minor version
        if bytes.remove(0) != Mail::VERSION_MINOR {
            panic!("Bad minor version number for message");
        };

        // Read account
        let next = bytes.remove(0);
        let account = match next {
            0    => None,
            size => {
                let acc: Vec<u8> = bytes.drain(0..size as usize).collect();
                let acc = str::from_utf8(acc.as_slice()).unwrap();
                Some(acc.to_string())
            }
        };

        // Read recipients
        let mut recipients = Vec::new();
        let mut next = bytes.remove(0);
        while 0 != next {
            let recipient: Vec<u8> = bytes.drain(0..next as usize).collect();
            let recipient = str::from_utf8(recipient.as_slice()).unwrap();
            recipients.push(recipient.to_string());
            next = bytes.remove(0);
        }

        // Read the body of the message
        let body = bytes.to_owned();

        Mail {
            account,
            recipients,
            body,
        }
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
        assert_eq!(expected, actual);
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
        assert_eq!(expected, actual);
    }
}
