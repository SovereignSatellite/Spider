-- SECTION import_map
local rt_import_map = {}

-- SECTION import
-- NEEDS import_map
local function rt_import(namespace, identifier)
	local space = rt_import_map[namespace]

	return assert(space and space[identifier], "`" .. namespace .. "." .. identifier .. "` should be registered")
end

-- SECTION export_map
local rt_export_map = {}

-- SECTION export
-- NEEDS export_map
local function rt_export(identifier, value)
	rt_export_map[identifier] = value
end
