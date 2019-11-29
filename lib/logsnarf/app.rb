# frozen_string_literal: true

require "logsnarf"

module Logsnarf
  class App
    include Import[:logger, :instrumenter]

    attr_reader :logsnarf

    def initialize(**imports)
      super
      credentials_store = Credentials::Store.new
      @logsnarf = Logsnarf::Loader.new(credentials_store: credentials_store)
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
    ensure
      Aws.empty_connection_pools!
    end
  end
end
