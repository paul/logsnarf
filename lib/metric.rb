# frozen_string_literal: true

Metric = Struct.new(:name, :tags, :values, :timestamp, keyword_init: true) do
  def complete?
    name && timestamp && !tags.empty? && !values.empty?
  end
end
