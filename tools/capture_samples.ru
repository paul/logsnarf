# frozen_string_literal: true

run(lambda do |env|
  name = (Time.now.to_f * 1_000_000).to_i.to_s
  IO.copy_stream(env["rack.input"], "samples/#{name}.log")
  [204, [], ""]
end)
