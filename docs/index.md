# Chess engine dev notes

## What's this about?
I think the goal of this set of documents is to, hopefully, give a somewhat
self-contained overview of the pieces that go into developing a modern chess 
engine.

There are many sources out there already, but most of them are either
1) very limited in scope (Rustic book, viriwiki, expositor notes)
2) outdated (Chess programming wiki)
3) a poorly curated mess (Chess programming wiki)
4) Conversational (Discord)
5) Existing codebases (not always well documented)

I hope to keep these notes as up to date as I can, motivation permitting, as
new techniques become more widely adopted.

## Rough outline
### UCI
### FEN parsing
### Board representation
#### Bitboards
#### Mailbox
### Movegen
#### Perft
#### Legal/pseudo-legal
#### Magics
#### PEXT
### Negamax
#### Heuristics
### Quiescence search
#### Heuristics
### Move ordering
### Testing
#### SPRT
#### Books
