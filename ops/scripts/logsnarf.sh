#!/bin/bash

set -Eeuxo pipefail

install -o root -g root -m 644 -v -D /tmp/templates/dnf.conf /etc/dnf/dnf.conf

# dnf update --refresh -y
# dnf groupinstall -y "Development Tools"
dnf install -y gcc make glibc-devel \
  git \
  openssl-devel

groupadd logsnarf
adduser \
  -g logsnarf --no-user-group \
  --shell /usr/sbin/nologin \
  logsnarf
install -o logsnarf -g logsnarf -m 755 -v -D -d /opt/logsnarf

# Allow user to start systemd services at boot
loginctl enable-linger logsnarf

## Caddy
dnf install -y 'dnf-command(copr)'
dnf copr enable -y @caddy/caddy epel-9-$(arch)
dnf install -y caddy

# which caddy
# # setcap 'cap_net_bind_service=+ep' /usr/local/bin/caddy

install -o root -g root -m 644 -v -D /tmp/templates/Caddyfile /etc/caddy/Caddyfile
install -o root -g root -m 644 -v -D /tmp/templates/sysctl-caddy.conf /etc/sysctl.d/50-caddy.conf
install -o root -g logsnarf -m 0770 -v -d /etc/ssl/caddy

# Copy letsencrypt certs in place to bootstrap caddy. TODO store these in s3/consul/do somehow
# find /tmp/templates/acme -type f -exec install -o logsnarf -g logsnarf -m 600 -v -D "{}" "/var/www/.local/share/caddy/acme/{}" \;

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
dnf install -y ruby-devel

install -o root -g root -m 644 -v -D /tmp/templates/gemrc /etc/gemrc
gem install bundler
# gem update --system

# # try installing some gems
# gem install io-event
# gem install ox
# gem install async

install -o root -g root -m 644 /tmp/templates/logsnarf.service /etc/systemd/system/logsnarf.service
install -o root -g root -m 644 /tmp/templates/restart-logsnarf.service /etc/systemd/system/restart-logsnarf.service
install -o root -g root -m 644 /tmp/templates/restart-logsnarf.timer /etc/systemd/system/restart-logsnarf.timer
systemctl daemon-reload
systemctl enable logsnarf.service
systemctl enable restart-logsnarf.timer

## Cleanup
dnf clean all && rm -rf /var/cache/dnf

