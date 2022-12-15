# frozen_string_literal: true

require_relative "system/boot"
require "logsnarf/app"

use Logsnarf::App[:rack_monitor]
run Logsnarf::App.new
