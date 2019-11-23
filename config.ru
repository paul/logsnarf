# frozen_string_literal: true

require "raven"
require "bundler/setup"
require "logsnarf/app"

Raven.configure do |config|
  config.dsn = ENV["SENTRY_DSN"]
end

use Raven::Rack
use Logsnarf::App

run lambda { |_env| [404, {}, []] }
