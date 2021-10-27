
alias TagKey = String
alias TagValue = String

alias FieldKey = String
alias FieldValue = Bool | Float64 | Int64 | String

alias Tags = Hash(TagKey, TagValue)
alias Fields = Hash(FieldKey, FieldValue)

class Metric
  property! name : String
  property! timestamp : Time
  property! tags : Tags
  property! fields : Fields
end

