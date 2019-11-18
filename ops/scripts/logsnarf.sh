#!/bin/bash

set -ex

wget -q https://github.com/caddyserver/caddy/releases/download/v2.0.0-beta9/caddy2_beta9_linux_amd64
install -o root -g root -m 755 -v caddy2_beta9_linux_amd64 /usr/local/bin/caddy

sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/caddy

sudo groupadd logsnarf
sudo useradd \
  -g logsnarf --no-user-group \
  --home-dir /var/www --no-create-home \
  --shell /usr/sbin/nologin \
  --system logsnarf

install -o root -g root -m 644 -v -D /tmp/templates/Caddyfile /etc/caddy2/Caddyfile
install -o root -g logsnarf -m 0770 -v -d /etc/ssl/caddy2
install -o logsnarf -g logsnarf -m 555 -v -D -d /var/www

# cp -R example.com /var/www/
# chown -R www-data:www-data /var/www/example.com
# chmod -R 555 /var/www/example.com

install -o root -g root -m 644 /tmp/templates/caddy2.service /etc/systemd/system/caddy2.service

systemctl daemon-reload
systemctl enable caddy2.service
systemctl start caddy2.service

# dnf install -y https://copr-be.cloud.fedoraproject.org/results/psadauskas/ruby-install/fedora-29-x86_64/01028211-ruby-install/ruby-install-0.7.0-1.noarch.rpm
# dnf config-manager --enable PowerTools # libyaml-devel

cd /var/www

# su scalar <<SETUP_APP
# set -ex
# git clone https://${GITHUB_TOKEN}@github.com/paul/scalarapp.git scalar
# cd scalar
# git checkout origin/production

# bundle install --without "development test" --path vendor/bundle --binstubs vendor/bundle/bin -j4 --deployment
# SETUP_APP

install -o root -g root -m 644 /tmp/templates/logsnarf.service /etc/systemd/system/logsnarf.service
systemctl enable logsnarf.service

