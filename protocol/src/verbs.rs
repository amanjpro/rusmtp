// Shamelessly taken from:
// http://smtpfilter.sourceforge.net/esmtp.html

pub const AUTH: &str = "AUTH";
pub const DATA: &str = "DATA";
pub const EHLO: &str = "EHLO";
pub const HELO: &str = "HELO";
pub const QUIT: &str = "QUIT";
pub const VRFY: &str = "VRFY";
pub const MAIL: &str = "MAIL";
pub const NOOP: &str = "NOOP";
pub const STARTTLS: &str = "STARTTLS";
pub const RCPT: &str = "RCPT";
pub const TO: &str = "TO";
pub const FROM: &str = "FROM";
pub const LOGIN: &str = "LOGIN";
pub const XOAUTH2: &str = "XOAUTH2";
pub const RSET: &str = "RSET";
