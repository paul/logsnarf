# frozen_string_literal: true

require "dry-types"

module Types
  include Dry.Types()

  URL_REGEXP = URI::DEFAULT_PARSER.make_regexp(["http", "https"])

  StrippedString = Types::Coercible::String.constructor(&:strip)
  URL = StrippedString.constrained(format: URL_REGEXP)
end
