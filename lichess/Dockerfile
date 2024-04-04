FROM alpine

ARG VERSION

RUN apk add --no-cache git wget py3-pip py3-virtualenv

RUN git clone https://github.com/lichess-bot-devs/lichess-bot

WORKDIR ./lichess-bot

RUN python3 -m venv venv && \
  virtualenv venv -p python3 && \
  . ./venv/bin/activate && \
  python3 -m pip install -r requirements.txt

COPY config.yml .

RUN wget \
 https://github.com/sroelants/simbelmyne/releases/download/$VERSION/simbelmyne-${VERSION}-x86_64-v3 \
 -O ./engines/simbelmyne && \
 chmod +x ./engines/simbelmyne

CMD . ./venv/bin/activate && python3 lichess-bot.py -v
