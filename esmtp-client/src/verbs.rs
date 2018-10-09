// Shamelessly taken from:
// http://smtpfilter.sourceforge.net/esmtp.html

pub const AUTH: &'static str = "AUTH";
pub const DATA: &'static str = "DATA";
pub const EHLO: &'static str = "EHLO";
pub const MAIL: &'static str = "MAIL";
pub const NOOP: &'static str = "NOOP";
pub const STARTTLS: &'static str = "STARTTLS";
pub const RCPT: &'static str = "RCPT";
pub const TO: &'static str = "TO";
pub const FROM: &'static str = "FROM";
pub const LOGIN: &'static str = "LOGIN";
pub const XOAUTH2: &'static str = "XOAUTH2";
pub const RSET: &'static str = "RSET";
