[![Build Status](https://travis-ci.org/amanjpro/rusmtp.svg?branch=master)](https://travis-ci.org/amanjpro/rusmtp)

# SMTP daemon

A simple and secure SMTP-client daemon. 

A typical scenario of using `rusmtp` is with clients like `NeoMutt`, where you
do not store your passwords unencrypted, but you still wish to get all the
benefits of using `NeoMutt`.

A typical installation would be as follows:

- Add `set sendmail="/PATH/TO/rusmtp"` in muttrc.
- Using GnuGP, encrypt your password and save the encrypted password.
- Add `passwordeval=gpg --quiet --no-tty --decrypt /PATH/TO/ENCRYPTED-PASSWORD.gpg`
  to rusmtprc for each account.
- Add the password to decrypt the encrypted password in gpg-agent, to avoid
  entering the password upon starting the deamon.
- Make rusmtpd to startup upon boot.

At its current state, the builtin SMTP client only supports ESMTP and
only supports LOGIN (i.e. it uses username and password to authenticate
the connection).

## Building from the source

`rusmtp` is written in rust, and it can be built with `cargo`, to build it simply
run `cargo build --release` and have the daemon built for device.

## Installation

- Download the latest release
  [here](https://github.com/amanjpro/rusmtp/releases), extract it and run
  `sudo ./install`, it copies the executables to `/usr/local/bin/{rusmtpc,rusmtpd}`
- Update the `~/.rusmtprc` file to match your preferences, for example
  the passwordeval setting can be:
  `passwordeval=gpg --quiet --no-tty --decrypt /PATH/TO/ENCRYPTED-PASSWORD.gpg`
- Update your email-client configuration to use `/usr/local/bin/rusmtpc` for
  sending emails.
- Make the `/usr/local/bin/rusmtpd` daemon to run on startup.
