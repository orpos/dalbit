--!strict

local argparse = require("submodules/argparse")
local process = require("@lune/process")

local dal = argparse("dal")
local transpile = dal:command("transpile", "Transpiles input luau sources into lua")

transpile:option("--input -i", "Input file or directory")
transpile:option("--output -o", "Output file or directory")
transpile:action(function(args: {
	input: string?,
	output: string?
})
	process.spawn("darklua")
end)

dal:parse()
