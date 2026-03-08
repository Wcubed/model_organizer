extends RefCounted

static func LoadFromPath(path: String) -> Variant:
	'''
	:return: ArrayMesh or Int
	'''
	var result := StlLoader.load_from_file(path)
	return result
