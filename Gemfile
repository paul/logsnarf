# frozen_string_literal: true

source "https://rubygems.org"
ruby File.read(".ruby-version").strip

gem "falcon", "~> 0.42.3"

gem "console", "~> 1.15"
gem "zeitwerk", "~> 2.6"

gem "aws-sdk-dynamodb", "~> 1.77"

gem "dry-events", "~> 0.3.0"
gem "dry-monads", "~> 1.4"
gem "dry-monitor", "~> 0.6.3"
gem "dry-struct", "~> 1.4"
gem "dry-system", "~> 0.25.0"

gem "sentry-ruby", "~> 5.5"

group :development do
  gem "debug", "~> 1.6"

  gem "bundler"
  gem "rake", "~> 13.0"

  gem "bcrypt_pbkdf", "~> 1.1"
  gem "capistrano", "~> 3.17", require: false
  gem "capistrano-bundler", "~> 2.1", require: false
  gem "capistrano-sentry", "~> 0.4", require: false
  gem "ed25519", "~> 1.3"

  gem "benchmark-ips", "~> 2.10"
  gem "get_process_mem", "~> 0.2"
  gem "memory_profiler", "~> 1.0"
  gem "mwrap", "~> 2.3"
  gem "ruby-prof", "~> 1.4"
  gem "syslog-parser", "~> 0.1"
end

group :development, :test do
  gem "rspec", "~> 3.11"

  gem "async-rspec", "~> 1.16"

  gem "reek",                "~> 6.1"
  gem "rubocop",             "~> 1.36"
  gem "rubocop-performance", "~> 1.15"
  gem "rubocop-rspec",       "~> 2.13"

  gem "amazing_print", "~> 1.4"
end
