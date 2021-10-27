
struct LogData 
  property! timestamp_str : String
  property! hostname : String
  property! appname : String
  property! procid : String
  property! msgid : String | Nil
  property! msg : String

  def initialize(@timestamp_str, @hostname, @appname, @procid, @msgid, @msg)

  end
end

class FastParser
  BLANK = '-'.ord
  SPACE = ' '.ord

  class ParseError < Exception
  end

  class UnexpectedEndOfInput < ParseError
  end

  def self.parse(line : String) : LogData?
    parse(line.to_slice)
  end

  def self.parse(line : Slice(UInt8)) : LogData?
    rest = line 

    rest = skip_to_after('>', rest)
    rest = skip_to_after(' ', rest)

    timestamp_str, rest = parse_term(rest)
    hostname, rest = parse_term(rest)
    appname, rest = parse_term(rest)
    procid, rest = parse_term(rest)
    msgid, rest = parse_term(rest)
    msg = String.new(rest)

    LogData.new(
      timestamp_str,
      hostname,
      appname,
      procid,
      msgid,
      msg)

  rescue ex : ParseError
    nil
  rescue ex : IndexError
    nil
  end 

  def self.skip_to_after(char, slice) : Slice(UInt8)
    ord = char.ord
    slice.each_with_index do |c, i|
      if c == ord
        return slice[i+1..]
      end
    end

    # raise UnexpectedEndOfInput.new("Failed to parse #{String.new(slice)}")
    Slice(UInt8).empty
  end

  def self.parse_term(slice) : {String?, Slice(UInt8)}
    # Blank field
    if slice[0] == BLANK && (slice.size <= 1 || slice[1] == SPACE)
      return {nil, slice[2..]}
    end

    # Read until we get a space
    slice.each_with_index do |c, i|
      if c == SPACE
        return {String.new(slice[0, i]), slice[i+1..]}
      end
    end

    # raise UnexpectedEndOfInput.new("Failed to parse #{String.new(slice)}")
    {nil, Slice(UInt8).empty}
  end
end
