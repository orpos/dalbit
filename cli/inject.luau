local pathfs = require("../submodules/dirs/submodules/pathfs")

local COMMENT_DIRECTIVE_PATTERN = "^%-%-!(.+)"

--- Injects a library into a script
return function(filePath: pathfs.Path, name: string, libraryPath: pathfs.Path?)
	if name:find(" ") then
		error("Blank space in library name is invalid")
	end
	local parent = filePath:parent()
	if not parent then
		error("filePath should have parent path")
	end

	local lines = pathfs.readFile(filePath):split("\n")
	local targetLine = #lines

	for i, line in lines do
		if not line:match(COMMENT_DIRECTIVE_PATTERN) then
			targetLine = i
			break
		end
	end

	local targetContent = lines[targetLine]
	local expression = if libraryPath then `require'{pathfs.diff(libraryPath, parent, "/")}'`
		else "nil"
	local requireAssign = `local {name}={expression} `
	lines[targetLine] = requireAssign .. targetContent

	pathfs.writeFile(filePath, table.concat(lines, "\n"))
end
