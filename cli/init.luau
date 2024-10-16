--!strict

local argparse = require("../submodules/argparse")
local process = require("@lune/process")
local config = require("./config")
local dirs = require("../submodules/dirs")
local pathfs = require("../submodules/dirs/submodules/pathfs")
local stdio = require("@lune/stdio")
local getStdout = require("./getStdout")

local function wrap(fn: (args: { [any]: any }) -> ())
	return function(args)
		local execPath = args.path
		if execPath then
			pathfs.setCurrentDir(execPath)
		end
		fn(args)
	end
end

local dal = argparse("dal")
dal:option("-C --path", "Execution path")

local transpile = dal:command("transpile", "Transpiles input luau sources into lua")
transpile:argument("input", "Input file or directory"):args("?")
transpile:argument("output", "Output file or directory"):args("?")
transpile:action(wrap(function(args: {
	input: string?,
	output: string?
})

end))

local init = dal:command("init", "Initialize a new dal configuration file")
init:option("--output", "Output path")
init:action(wrap(function(args: { output: string? })
	local configPath = pathfs.Path.from("dal.toml")
	if pathfs.isFile(configPath) then
		error(`Configuration file already exists in '{pathfs.getAbsolutePathOf(configPath)}'`)
	end
	local output = if args.output then args.output else stdio.prompt("text", "Enter your preferred output path (optional)")
		pathfs.writeFile(configPath, `{if output ~= "" then "output = \"" .. output .. "\"\n\n" else ""}# [rules] # Configures darklua rules.\
# remove_generalized_iteration = false\
\
# [libs] # Configures whether libraries are inserted.\
# buffer = false\
`)
	print(`Successfully initialized dal configuration file at '{pathfs.getAbsolutePathOf(configPath)}'`)
end))

dal:parse()
