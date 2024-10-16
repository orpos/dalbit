local process = require("@lune/process")

return function(command: string, args: { string }): string
	local result = process.spawn(command, args)
	if result.ok then
		return result.stdout
	end
	error(`Error while spawning command '{command}'\nstderr: {result.stderr}\nstdout: {result.stdout}`)
end
