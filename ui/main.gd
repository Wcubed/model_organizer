extends PanelContainer

@onready var folder_dialog := $FolderDialog
@onready var model_cards := %ModelCards

var model_card_scene := preload("res://ui/model_card.tscn")

## List of extensions that will be ignored when checking if a folder contains models.
var ignored_extensions := PackedStringArray([".zip", ".rar", ".orynt3d"])
var library_dir: String = ""
var model_dirs := PackedStringArray()

func _ready() -> void:
	scan_library()

func scan_library():
	if library_dir.is_empty():
		refresh_model_cards()
		return
	
	var found_model_dirs := PackedStringArray()
	scan_directory(library_dir, found_model_dirs)
	
	model_dirs = found_model_dirs
	
	refresh_model_cards()

func scan_directory(path: String, found_model_dirs: PackedStringArray):
	var dir = DirAccess.open(path)
	if !dir:
		return
	
	# Directory exists. Scan it.
	var files = Array(dir.get_files())
	files = files.filter(_filter_ignored_files)
	
	if files.size() > 0:
		# Model directory
		found_model_dirs.append(path)
	else:
		# Might contain sub directories
		var subdirs = dir.get_directories()
		for subdir in subdirs:
			var new_path = "%s/%s" % [path, subdir]
			scan_directory(new_path, found_model_dirs)

## Clears the model cards and displays the currently found models.
func refresh_model_cards():
	for card in model_cards.get_children():
		model_cards.remove_child(card)
		card.queue_free()
	
	for model in model_dirs:
		var new_card = model_card_scene.instantiate()
		model_cards.add_child(new_card)

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
