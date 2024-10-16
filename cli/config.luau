local TomlStruct = require("../submodules/TomlStruct")

export type Config = {
	output: string?,
	rules: {
		[string]: boolean
	}?,
	libs: {
		[string]: boolean
	}?
}

local config = TomlStruct(
	{ "output", "string" },
	{},
	{ "rules" },
	{},
	{ "libs" }
) :: TomlStruct.TomlStruct<Config>

return config
