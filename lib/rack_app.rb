# frozen_string_literal: true

class RackApp
  include Import[:application]
  include Dry::Monads[:result]

  INGRESS = "ingress"
  HEADERS = [].freeze
  BODY = [""].freeze
  def call(env)
    _, endpoint, token = *env[Rack::PATH_INFO].split("/", 3)

    return [404, HEADERS, BODY] unless endpoint == INGRESS

    case application.ingress(token, env[Rack::RACK_INPUT])
    in Success(_) | Success()
      [204, HEADERS, BODY]
    in Failure(:credentials)
      [403, HEADERS, BAD_CREDENTIALS]
    in Failure(_)
      [400, HEADERS, BODY]
    end
  ensure
    Aws.empty_connection_pools!
  end
end
