extends PanelContainer

## Emitted when the background library scan is complete.
signal library_scan_complete(found_models: Array[Model])

var scan_thread: Thread = null
## Updated by the background thread, read by the main thread.
var amount_found := 0

## List of extensions that will be ignored when checking if a folder contains models.
var ignored_extensions := PackedStringArray([".zip", ".rar", ".orynt3d", ".md", ".txt"])

func _process(_delta: float) -> void:
	if scan_thread == null:
		# Nothing to do here.
		return
	
	%AmountLabel.text = "%d" % amount_found
	
	if !scan_thread.is_alive():
		var found_models = scan_thread.wait_to_finish()
		library_scan_complete.emit(found_models)
		# Get out of the users way.
		hide()
		scan_thread = null

## Starts the library scan in the background.
func background_scan_library(library_dir: String):
	show()
	amount_found = 0
	%AmountLabel.text = ""
	
	scan_thread = Thread.new()
	scan_thread.start(_thread_scan_library.bind(library_dir))


func _thread_scan_library(library_dir: String) -> Array[Model]:
	var found_models: Array[Model] = []
	_scan_directory(library_dir, found_models)
	found_models.sort_custom(_sort_models_by_name)
	return found_models


func _scan_directory(path: String, found_models: Array[Model]):
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
		amount_found = found_models.size()
	else:
		# Might contain sub directories
		var subdirs = dir.get_directories()
		
		# First check if this is a "thingyverse" folder structure.
		for subdir in subdirs:
			if subdir.to_lower() == "files":
				# Thingyverse.
				var new_model = Model.new(path)
				new_model.scan_directory()
				found_models.append(new_model)
				amount_found = found_models.size()
				return
		
		for subdir in subdirs:
			var new_path = "%s/%s" % [path, subdir]
			_scan_directory(new_path, found_models)


## Returns true if the file should be included, false otherwise.
func _filter_ignored_files(file: String) -> bool:
	for extension in ignored_extensions:
		if file.to_lower().ends_with(extension):
			return false
	
	return true


func _sort_models_by_name(a: Model, b: Model) -> bool:
	return Utils.sort_string_natural_order(a.name, b.name)
