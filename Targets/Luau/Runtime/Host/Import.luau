-- needs Imports
const function import(namespace: string, identifier: string): any
	return imports[namespace][identifier]
end
