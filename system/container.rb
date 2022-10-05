# frozen_string_literal: true

require "dry/events"
require "dry/monitor/notifications"
require "dry/system/container"

class App < Dry::System::Container
  use :env, inferrer: -> { ENV.fetch("RACK_ENV", :development).to_sym }
  use :zeitwerk, debug: true
  use :logging
  use :monitoring

  configure do |config|
    config.component_dirs.add "lib"

    config.logger = Console.logger
  end
end
