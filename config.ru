# frozen_string_literal: true

require "bundler/setup"
require "logsnarf/app"

use Logsnarf::App

run lambda { |_env| [404, {}, []] }
