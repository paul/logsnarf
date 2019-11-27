# frozen_string_literal: true

require "logsnarf"

module Logsnarf
  class App
    attr_reader :logsnarf

    def initialize(app, logger: Async.logger)
      @app = app
      adapter = Adapter::InfluxdbV2.new(ENV["INSTRUMENTER_URL"], logger: logger, instrumenter: nil)
      instrumenter = NullInstrumenter.new(adapter: adapter)
      credentials_store = Credentials::Store.new(logger: logger)
      @logsnarf = Logsnarf::Loader.new(logger: logger, instrumenter: instrumenter, credentials_store: credentials_store)
    end

    def call(env)
      _, endpoint, token = *env["PATH_INFO"].split("/", 3)

      if endpoint == "ingress"
        @logsnarf.load(token, env["rack.input"])
        [204, [], ""]
      else
        [404, [], ""]
      end
    rescue Logsnarf::AuthError => e
      [403, [], "Who the hell are you?"]
      # rescue StandardError => e
      #   # raise e

      #   # Heroku logdrain stops sending if you return too many errors or take to
      #   # long, so don't raise anything
      #   [202, [], ""]
    ensure
      Aws.empty_connection_pools!
    end
  end
end
