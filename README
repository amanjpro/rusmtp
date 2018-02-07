# SMTP daemon

A Simple SMTP daemon to facilitate working with encrypted passwords.

SMTP clients by default do not maintain the connection with the server.
If you don't like to enter your password whenever you send an email,
or the fact that your email/SMTP client stores passwords in plain text,
then this is the right tool for you.

This consists of two parts, a server that tries to get the password upon
starting (maybe decrypting it from gpg?), and a client that sends the
email to be sent to the server.

The server then launches an SMTP client (configurable) and passes the
unencrypted password to it.

# Installation

- Clone the repo, and run `sudo ./install`, this installs the binaries
  on `/usr/local/bin/{smtpc,smtpd}`. You need rust stable to perform
  this step.

- Update the ~/smtpdrc file to match your preferences, for example
  the passwordeval setting can be:
  `passwordeval=gpg --quiet --no-tty --decrypt /path/to/encrypted-password.gpg`
- Update your client configuration to use `/usr/local/bin/smtpc` for
  sending emails.
- Make the `/usr/local/bin/smtpd` daemon to run on startup.
