# frozen_string_literal: true

require "logsnarf"
require "raven"

Raven.configure do |config|
  config.dsn = "https://48fff5c713d34c0792976ed30896764e@sentry.io/1823509"
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
      ap e
      # Raven.capture_exception(e)

      raise e unless ENV["RACK_ENV"] = "production"

      # Heroku logdrain stops sending if you return too many errors or take to
      # long, so don't raise anything
      [202, [], ""]
    end
  end
end
