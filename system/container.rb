# frozen_string_literal: true

require "dry/events"
require "dry/monads"
require "dry/monads/do"
require "dry/monitor/notifications"
require "dry/system/container"
require "amazing_print"

class App < Dry::System::Container
  use :env, inferrer: -> { ENV.fetch("RACK_ENV", :development).to_sym }
  use :zeitwerk, debug: false
  use :logging
  use :monitoring

  configure do |config|
    config.component_dirs.add "lib" do |dir|
      dir.instance = ->(component) {
        case component.key
        when /adapters\./, /clients\./
          component.loader.constant(component)
        else
          component.loader.call(component)
        end
      }
    end
    config.logger = Console.logger
  end
end
