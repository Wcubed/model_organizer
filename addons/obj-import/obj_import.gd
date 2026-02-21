@tool
extends ResourceFormatLoader
class_name OBJ_IMPORT
## Made by Wyanne

const Importer = preload('res://addons/obj-import/obj_parse.gd')

static func RegisterFormatLoader(at_front := false) -> void:
	if 'obj' in ResourceLoader.get_recognized_extensions_for_type('ArrayMesh'):
		return
	ResourceLoader.add_resource_format_loader(OBJ_IMPORT.new(), at_front)


func _exists(path: String) -> bool:
	'''
	Returns whether a recognized resource exists for the given path.
	'''
	return FileAccess.file_exists(path)

func _handles_type(type: StringName) -> bool:
	return type in [&'ArrayMesh', &'Resource']

func _get_recognized_extensions() -> PackedStringArray:
	return PackedStringArray(['obj'])

func _get_resource_script_class(_path: String) -> String:
	return 'ArrayMesh'

func _get_resource_type(path: String) -> String:
	if path.get_extension() in _get_recognized_extensions():
		return 'ArrayMesh'
	return ''

func _load(
	path: String,
	_original_path: String,
	_use_sub_threads: bool,
	_cache_mode: int
	) -> Variant:
	return Importer.from_path(path)

static func IsError(result :Variant) -> bool:
	return typeof(result) == TYPE_INT
