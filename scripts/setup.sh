#!/bin/bash

################################################################################
#
# Set sensitive and worker-specific parameters
#
################################################################################

OB_PASSWORD=
OB_NAME=
OB_THREADS=

if [ -z "$OB_PASSWORD" ]; then 
  echo "OB_PASSWORD variable not set"
  exit 1
fi

if [ -z "$OB_NAME" ]; then 
  echo "OB_NAME variable not set"
  exit 1
fi

if [ -z "$OB_THREADS" ]; then 
  echo "OB_THREADS variable not set"
  exit 1
fi

################################################################################
#
# Install dependencies
#
################################################################################

sudo apt-get update;
sudo apt-get install python3 pip curl git -y;

################################################################################
#
# Install latest rust toolchain
#
################################################################################

curl https://sh.rustup.rs -sSf >> rustup.sh 
chmod +x rustup.sh 
./rustup.sh -y;
source $HOME/.cargo/env;

################################################################################
#
# Check out OpenBench code
#
################################################################################

git clone https://github.com/sroelants/OpenBench;
cd OpenBench/Client/;

################################################################################
#
# Install python dependencies
#
################################################################################

pip install -r requirements.txt --break-system-packages;

################################################################################
#
# Generate run script
#
################################################################################

touch $(pwd)/run-ob.sh
bash -c "cat > $(pwd)/run-ob.sh <<'EOL'
#!/usr/bin/env bash

source $HOME/.cargo/env;
cd ./OpenBench/Client;
/usr/bin/python3 client.py -U "sroelants" -P $OB_PASSWORD -S "https://chess.samroelants.com" -I $OB_NAME -T $OB_THREADS -N 1
EOL";

chmod +x $(pwd)/run-ob.sh

################################################################################
#
# Generate systemd service
#
################################################################################

sudo touch /etc/systemd/system/openbench-worker.service;
sudo bash -c "cat > /etc/systemd/system/openbench-worker.service <<'EOL'
[Unit]
Description=OpenBench Worker
After=network.target

[Service]
Type=simple
StandardError=inherit
ExecStart=$(pwd)/run-ob.sh
User=$(whoami)
WorkingDirectory=$(pwd)
Restart=on-failure
RestartSec=600

[Install]
WantedBy=multi-user.target
EOL";

################################################################################
#
# Enable and install service
#
################################################################################

sudo systemctl daemon-reload;
sudo systemctl enable openbench-worker.service;
sudo systemctl start openbench-worker.service;

################################################################################
#
# Do a daily restart (what's up witht his, fury?)
# Probably just a robustness thing
#
################################################################################

sudo crontab -l > mycron;
echo "0 * * * * sudo systemctl restart openbench-worker" >> mycron;
sudo crontab mycron
