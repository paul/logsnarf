# frozen_string_literal: true

require "dry/system/container"
require "async"

module Logsnarf
  class App < Dry::System::Container
    use :env, inferrer: -> { ENV.fetch("RACK_ENV", :development).to_sym }
    use :logging
    use :notifications

    configure do
      config.name = :logsnarf
      config.default_namespace = "logsnarf"
      config.auto_register = %w[lib/logsnarf]
      config.logger = Async.logger
    end

    load_paths! "lib"
  end
end
