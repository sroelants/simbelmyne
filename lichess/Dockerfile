################################################################################
#
# Fetch latest main and build from source
#
################################################################################
FROM rust:1.77 as builder

RUN git clone https://github.com/sroelants/simbelmyne

WORKDIR ./simbelmyne

ENV RUSTFLAGS=-Ctarget-feature=+crt-static,-bmi2 -Ctarget-cpu=x86-64-v3
RUN rustup override set 1.73.0
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl --bin simbelmyne

################################################################################
#
# Download and start lichess-bot
#
################################################################################

FROM alpine

RUN apk add --no-cache git wget py3-pip py3-virtualenv

RUN git clone https://github.com/lichess-bot-devs/lichess-bot

WORKDIR ./lichess-bot

RUN python3 -m venv venv && \
  virtualenv venv -p python3 && \
  . ./venv/bin/activate && \
  python3 -m pip install -r requirements.txt

COPY config.yml .

COPY --from=builder ./simbelmyne/target/x86_64-unknown-linux-musl/release/simbelmyne ./engines/simbelmyne
RUN chmod +x ./engines/simbelmyne

CMD . ./venv/bin/activate && python3 lichess-bot.py -v
