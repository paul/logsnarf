# frozen_string_literal: true

require "dry-struct"

class Credentials < Dry::Struct
  extend Forwardable
  transform_keys(&:to_sym)

  attribute :name, Types::String
  attribute :token, Types::String
  attribute :created_at, Types::JSON::Time
  attribute :updated_at, Types::JSON::Time

  attribute :credentials do
    attribute :type, Types::String
    attribute :url, Types::URL
  end

  delegate [:type] => :credentials
end
