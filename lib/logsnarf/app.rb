# frozen_string_literal: true

require "logsnarf"

module Logsnarf
  class App
    def initialize(app)
      @app = app
    end

    def call(env)
      Logsnarf.parse(env["rack.input"].read)

      [202, [], ""]
    end
  end
end
