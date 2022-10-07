#!/bin/bash

set -Eeuxo pipefail

CENTOS=`rpm --eval '%{centos_ver}'`

install -o root -g root -m 644 -v -D /tmp/templates/dnf.conf /etc/dnf/dnf.conf

# dnf update --refresh -y

dnf groupinstall -y "Development Tools"
# dnf install -y git wget

groupadd logsnarf
useradd \
  -g logsnarf --no-user-group \
  --home-dir /var/www --no-create-home \
  --shell /usr/sbin/nologin \
  --system logsnarf

install -o logsnarf -g logsnarf -m 755 -v -D -d /var/www

## Caddy
# wget -q https://github.com/caddyserver/caddy/releases/download/v2.0.0-beta10/caddy2_beta10_linux_amd64
# install -o root -g root -m 755 -v caddy2_beta10_linux_amd64 /usr/local/bin/caddy2
dnf install -y 'dnf-command(copr)'
dnf copr enable -y @caddy/caddy epel-9-$(arch)
dnf install -y caddy

which caddy
# setcap 'cap_net_bind_service=+ep' /usr/local/bin/caddy

install -o root -g root -m 644 -v -D /tmp/templates/Caddyfile /etc/caddy/Caddyfile
install -o root -g logsnarf -m 0770 -v -d /etc/ssl/caddy

# Copy letsencrypt certs in place to bootstrap caddy. TODO store these in s3/consul/do somehow
find /tmp/templates/acme -type f -exec install -o logsnarf -g logsnarf -m 600 -v -D "{}" "/var/www/.local/share/caddy/acme/{}" \;

# install -o root -g root -m 644 /tmp/templates/caddy2.service /etc/systemd/system/caddy2.service

systemctl daemon-reload
systemctl enable caddy.service

## Telegraf
# wget https://dl.influxdata.com/telegraf/releases/telegraf-1.12.6-1.x86_64.rpm
# yum localinstall -y telegraf-1.12.6-1.x86_64.rpm

# install -o root -g root -m 644 -v -D /tmp/templates/telegraf.influxdb.conf /etc/telegraf/telegraf.d/influxdb.conf
# install -o root -g root -m 644 -v -D /tmp/templates/telegraf.net.conf /etc/telegraf/telegraf.d/net.conf

# systemctl daemon-reload
# systemctl enable telegraf.service

## Ruby
dnf module -y reset ruby
dnf module -y enable ruby:3.1
dnf module -y install ruby:3.1/common

which bundler
gem install bundler
# gem update --system

install -o root -g root -m 644 /tmp/templates/logsnarf.service /etc/systemd/system/logsnarf.service
install -o root -g root -m 644 /tmp/templates/restart-logsnarf.service /etc/systemd/system/restart-logsnarf.service
install -o root -g root -m 644 /tmp/templates/restart-logsnarf.timer /etc/systemd/system/restart-logsnarf.timer
systemctl daemon-reload
systemctl enable logsnarf.service
systemctl enable restart-logsnarf.timer

## Cleanup
dnf clean all && rm -rf /var/cache/dnf

