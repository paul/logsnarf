# frozen_string_literal: true

require "dry/system/container"
require "async"
require "console/serialized/logger"

module Logsnarf
  class App < Dry::System::Container
    use :env, inferrer: -> { ENV.fetch("RACK_ENV", :development).to_sym }
    use :logging
    use :notifications

    configure do
      config.name = :logsnarf
      config.default_namespace = "logsnarf"
      config.auto_register = %w[lib/logsnarf]
      config.logger = if config.env == :test
                        logger = Console::Logger.new(Console::Serialized::Logger.new(File.open("log/test.log", "a")))
                        Console.logger = logger
                        logger
                      else
                        Async.logger
      end
    end

    load_paths! "lib"
  end
end
