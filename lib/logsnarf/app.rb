# frozen_string_literal: true

require "logsnarf"
require "raven"

Raven.configure do |config|
  config.dsn = ENV["SENTRY_DSN"]
end

module Logsnarf
  class App
    def initialize(app)
      @app = app
      @logsnarf = Logsnarf::Loader.new
    end

    def call(env)
      _, endpoint, rest = *env["PATH_INFO"].split("/", 3)

      if endpoint == "ingress"
        @logsnarf.load(rest, env["rack.input"])
        [202, [], ""]
      else
        [404, [], ""]
      end
    rescue Logsnarf::AuthError => e
      [403, [], "Who the hell are you?"]
    rescue StandardError => e
      if ENV["RACK_ENV"] == "production"
        Raven.capture_exception(e)
      else
        raise e
      end

      # Heroku logdrain stops sending if you return too many errors or take to
      # long, so don't raise anything
      [202, [], ""]
    end
  end
end
