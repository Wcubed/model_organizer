extends PanelContainer

@onready var folder_dialog := $FolderDialog

## List of extensions that will be ignored when checking if a folder contains models.
var ignored_extensions := PackedStringArray([".zip", ".rar", ".orynt3d"])
var library_dir: String = ""

func _ready() -> void:
	scan_library()

func scan_library():
	var model_dirs := PackedStringArray()
	scan_directory(library_dir, model_dirs)
	
	print(model_dirs)

func scan_directory(path: String, model_dirs: PackedStringArray):
	var dir = DirAccess.open(path)
	if !dir:
		return
	
	# Directory exists. Scan it.
	var files = Array(dir.get_files())
	files = files.filter(_filter_ignored_files)
	
	if files.size() > 0:
		# Model directory
		model_dirs.append(path)
	else:
		# Might contain sub directories
		var subdirs = dir.get_directories()
		for subdir in subdirs:
			var new_path = "%s/%s" % [path, subdir]
			scan_directory(new_path, model_dirs)

## Returns true if the file should be included, false otherwise.
func _filter_ignored_files(file: String) -> bool:
	for extension in ignored_extensions:
		if file.ends_with(extension):
			return false
	
	return true

func _on_path_button_pressed() -> void:
	folder_dialog.show()


func _on_folder_dialog_dir_selected(dir: String) -> void:
	library_dir = dir
	folder_dialog.current_dir = dir
	scan_library()
