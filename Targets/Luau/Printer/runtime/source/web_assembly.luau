-- SECTION import_map
local rt_import_map: { [string]: { [string]: any } } = {}

-- SECTION import
-- NEEDS import_map
local function rt_import(namespace: string, identifier: string)
	local space = rt_import_map[namespace]

	return assert(space and space[identifier], "`" .. namespace .. "." .. identifier .. "` should be registered")
end

-- SECTION export_map
local rt_export_map: { [string]: any } = {}

-- SECTION export
-- NEEDS export_map
local function rt_export(identifier: string, value: any)
	rt_export_map[identifier] = value
end
