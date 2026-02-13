extends PanelContainer

@onready var amount_label := %AmountLabel
@onready var folder_dialog := $FolderDialog
@onready var model_cards := %ModelCards

var model_card_scene := preload("res://ui/model_card.tscn")

## List of extensions that will be ignored when checking if a folder contains models.
var ignored_extensions := PackedStringArray([".zip", ".rar", ".orynt3d"])
var config_file_name := "user://config.cfg"
var library_dir: String = ""
## All the models found in the library.
var models: Array[Model] = []

func _ready() -> void:
	load_settings()
	scan_library()

func save_settings():
	var config = ConfigFile.new()
	config.set_value("main", "library_dir", library_dir)
	
	config.save(config_file_name)


func load_settings():
	var config = ConfigFile.new()
	var err = config.load(config_file_name)
	if err != OK:
		return
	
	library_dir = config.get_value("main", "library_dir", "")


func scan_library():
	if library_dir.is_empty():
		refresh_model_cards()
		return
	
	var found_models: Array[Model] = []
	scan_directory(library_dir, found_models)
	
	found_models.sort_custom(_sort_models_by_name)
	models = found_models
	
	amount_label.text = "%s models" % models.size()
	
	refresh_model_cards()

func scan_directory(path: String, found_models: Array[Model]):
	var dir = DirAccess.open(path)
	if !dir:
		return
	
	# Directory exists. Scan it.
	var files = Array(dir.get_files())
	files = files.filter(_filter_ignored_files)
	
	if files.size() > 0:
		# This is a model directory
		var new_model = Model.new(path)
		new_model.scan_directory()
		found_models.append(new_model)
	else:
		# Might contain sub directories
		var subdirs = dir.get_directories()
		for subdir in subdirs:
			var new_path = "%s/%s" % [path, subdir]
			scan_directory(new_path, found_models)

## Clears the model cards and displays the currently found models.
func refresh_model_cards():
	for card in model_cards.get_children():
		model_cards.remove_child(card)
		card.queue_free()
	
	for model in models:
		var new_card = model_card_scene.instantiate()
		new_card.model = model
		model_cards.add_child(new_card)

## Returns true if the file should be included, false otherwise.
func _filter_ignored_files(file: String) -> bool:
	for extension in ignored_extensions:
		if file.ends_with(extension):
			return false
	
	return true


func _sort_models_by_name(a: Model, b: Model) -> bool:
	return a.name < b.name

func _on_path_button_pressed() -> void:
	folder_dialog.show()


func _on_folder_dialog_dir_selected(dir: String) -> void:
	library_dir = dir
	folder_dialog.current_dir = dir
	
	save_settings()
	scan_library()


func _on_reload_button_pressed() -> void:
	scan_library()
