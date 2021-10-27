
require "./metric"
require "./fast_parser"
# require "./decoder"

class App

  def extract(buffer : IO, adapter = nil)
    metrics = [] of LogData
    buffer.each_line do |line|
      metric = decode(line)
      metrics << metric if metric
    end
    puts metrics.size
  end

  def decode(line)
    data = parse(line)
    metric = extract(data) if data
    metric
    data
  end

  def parse(line)
    FastParser.parse(line)
  end

  def extract(data)
    # Decoder.extract(data)
  end
end
