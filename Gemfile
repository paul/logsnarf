# frozen_string_literal: true

source "https://rubygems.org"
ruby File.read(".ruby-version").strip

gem "dry-core"
gem "dry-events"
gem "dry-monitor", github: "paul/dry-monitor", branch: "payload-enhancements"
gem "dry-system"

gem "aws-sdk-dynamodb"
gem "lru_redux"

gem "async-http"

gem "falcon"

gem "sentry-raven"

gem "awesome_print"

group :development do
  gem "pry"
  gem "pry-byebug"
  gem "pry-doc"

  gem "async-rspec"
  gem "bundler", "~> 2.0"
  gem "rake", "~> 10.0"
  gem "rspec", "~> 3.0"

  gem "bcrypt_pbkdf"
  gem "capistrano", require: false
  gem "capistrano-bundler", require: false
  gem "capistrano-sentry", require: false
  gem "ed25519"

  gem "benchmark-ips"
  gem "get_process_mem"
  gem "memory_profiler"
  gem "mwrap"
  gem "ruby-prof"
  gem "syslog-parser"
end

gemspec
