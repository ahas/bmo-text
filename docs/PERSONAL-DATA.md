# Personal data

No personal data stands in this repository. It is not written into code, comments, documents, test
fixtures or commit messages, and no example in them is a real value.

## What it is

- A person's name, handle, account id or email address.
- A host name, a domain, a home path, a device or a network address, a private address.
- A room member's display name and id.
- A company's name.
- A line of a real conversation.

## What to write instead

- A value that differs per deployment, per person or per room is configuration the consumer passes in;
  it is never a constant here.
- An example is neutral: a role such as `tester`, a made-up line, never a real name, account or address,
  and never a line someone actually wrote.
- A form the check finds is not written even with a neutral value. An email address, a home path, a phone
  number, a token and a private address are configuration the consumer supplies, or left out.

## The values the owner allows

Two values stand in this repository because they name its owner and maintainer: `ahas`, and the email
address `dev@ahas.io`. The check admits them and no other personal value.

## The check

The repository test reads every file of the checkout and fails where it finds an email address, a home
path, a phone number, a token or a private address. It reports the file, the line and the kind, never the
value it found. The WebAssembly build's encoded bytes are not scanned: they are generated, and a long run
of encoded bytes is no one's writing.

The repository holds no list of the words that name a person or a company. The check finds the forms
above; a writer follows the rule itself.
