
require "string_scanner"

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

class Parser
  SPACE = " " # TODO is this faster?
  ANGLE = ">"

  property! line : String

  def self.parse(line)
    new(line).parse
  end

  def initialize(@line : String)
  end

  def parse : LogData?
    s = StringScanner.new(line)
    s.skip_until(/>/)
    s.skip_until(/ /)

    timestamp_str = s.scan_until(/ /)
    hostname = s.scan_until(/ /)
    appname = s.scan_until(/ /)
    procid = s.scan_until(/ /)
    msgid = s.scan_until(/ /)
    msg = s.rest

    return unless timestamp_str && hostname && appname && procid && msg

    LogData.new(
      timestamp_str.strip,
      hostname.strip,
      appname.strip,
      procid.strip,
      msgid.try(&.strip),
      msg)
  end
end
