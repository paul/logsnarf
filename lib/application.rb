# frozen_string_literal: true

class Application
  include Import[:credentials_store]

  INGRESS = "ingress"
  def call(env)
    _, endpoint, token = *env[Rack::PATH_INFO].split("/", 3)

    if endpoint == INGRESS
      credentials_store.get(token)
      # @logsnarf.load(token, env[Rack::RACK_INPUT])
      [204, [], [""]]
    else
      [404, [], [""]]
    end
  # rescue Logsnarf::AuthError => _e
  #   [403, [], "Who the hell are you?"]
  ensure
    Aws.empty_connection_pools!
  end
end
