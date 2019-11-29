# frozen_string_literal: true

require "bundler/setup"

begin
  require "pry-byebug"
rescue LoadError
  nil
end

require "ap"

require_relative "container"

Logsnarf::App.finalize!

require "logsnarf/app"
