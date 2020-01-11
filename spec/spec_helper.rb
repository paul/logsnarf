# frozen_string_literal: true

ENV["RACK_ENV"] = "test"

require_relative "../system/boot"
require "logsnarf"

# require "dry/system/stubs"
# Logsnarf::App.enable_stubs!

require "async/rspec"

RSpec.configure do |config|
  # Enable flags like --only-failures and --next-failure
  config.example_status_persistence_file_path = ".rspec_status"

  # Disable RSpec exposing methods globally on `Module` and `main`
  config.disable_monkey_patching!

  config.expect_with :rspec do |c|
    c.syntax = :expect
  end
end

def base
  @base ||= Pathname.pwd.join("spec/fixtures")
end

def log_sample(sample)
  all_log_fixtures[sample.to_s + ".log"]
end

def other_samples(without)
  all_log_fixtures.reject { |k, _v| k == without.to_s + ".log" }
end

def all_log_fixtures
  @all_log_fixtures ||= base.glob("*.log").map { |f| [f.relative_path_from(base).to_s, f.read] }.to_h
end
