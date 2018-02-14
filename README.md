# SMTP daemon

A Simple SMTP daemon to facilitate working with encrypted passwords.

SMTP clients by default do not maintain the connection with the server.
If you don't like to enter your password whenever you send an email,
or the fact that your email/SMTP client stores passwords in plain text,
then this is the right tool for you.

This consists of two parts, a server that tries to get the password upon
starting (maybe decrypting it from gpg?), and a client that sends the
email to be sent to the server.

By default the daemon uses a builtin SMTP client that maintains the connection
with the SMTP server, does not keep the unencrypted password in memory. The
builtin SMTP client uses a secure connection (TLS) whenever available. If you
are sick with the builtin client or you are fine to keep the password
unencrypted in the memory, then you can configure the daemon to launch a
third-party SMTP client without losing any convenience.

At its current state, the builtin SMTP client only supports ESMTP and
only supports LOGIN (i.e. it uses username and password to authenticate
the connection).

## Installation

- Download the latest release
  [here](https://github.com/amanjpro/smtp-daemon/releases), extract it and run
  `sudo ./install`, it copies the executables to `/usr/local/bin/{smtpc,smtpd}`
- Update the `~/.smtpdrc` file to match your preferences, for example
  the passwordeval setting can be:
  `passwordeval=gpg --quiet --no-tty --decrypt /path/to/encrypted-password.gpg`
- Update your email-client configuration to use `/usr/local/bin/smtpc` for
  sending emails.
- Make the `/usr/local/bin/smtpd` daemon to run on startup.
