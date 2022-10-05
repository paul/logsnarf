# frozen_string_literal: true

require "bundler/setup"

require_relative "system/container"
require_relative "system/import"

App.finalize!

use App[:rack_monitor]
run App[:app]
