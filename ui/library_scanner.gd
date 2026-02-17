extends PanelContainer

## Emitted when the background library scan is complete.
signal library_scan_complete(found_models: Array[Model])

var scan_thread: Thread = null

## After being detected, the models are scanned in multiple threads.
var model_scan_threads: Array[Thread] = []
var model_scan_queue: Array[Model] = []
## After being processed, the models are appended to the found models array.
## We cannot create and delete references to the Model in multiple threads
## at the same time, because RefCounted _isnt_ atomic!
## So we make the model scan thread responsible for handling the references.
var found_models: Array[Model] = []
var run_model_scan_thread := false

## Updated by the background thread, read by the main thread.
var amount_found := 0
## Updated by the background thread, read by the main thread.
var amount_processed := 0
var time_scan_started := 0.0

## List of extensions that will be ignored when checking if a folder contains models.
var ignored_extensions := PackedStringArray([".zip", ".rar", ".orynt3d", ".md", ".txt", ".moa"])

func _process(_delta: float) -> void:
	if scan_thread == null:
		# Nothing to do here.
		return
	
	var time_taken := Time.get_unix_time_from_system() - time_scan_started
	
	%AmountLabel.text = "%d/%d" % [amount_processed, amount_found]
	%ProgressBar.value = amount_processed as float / amount_found as float
	
	if !scan_thread.is_alive() && model_scan_queue.is_empty():
		run_model_scan_thread = false
		scan_thread.wait_to_finish()

		for thread in model_scan_threads:
			thread.wait_to_finish()
		
		library_scan_complete.emit(found_models)
		print("Scan took %.1fs" % time_taken)
		
		# Get out of the users way.
		hide()
		scan_thread = null

## Starts the library scan in the background.
func background_scan_library(library_dir: String):
	show()
	found_models.clear()
	amount_found = 0
	amount_processed = 0
	%AmountLabel.text = ""
	%ProgressBar.value = 0
	
	time_scan_started = Time.get_unix_time_from_system()
	
	scan_thread = Thread.new()
	scan_thread.start(_thread_scan_library.bind(library_dir))
	
	run_model_scan_thread = true
	
	# Arbitrary number of threads.
	for i in 1:
		var new_thread := Thread.new()
		new_thread.start(_thread_scan_queued_models)
		model_scan_threads.append(new_thread)


func _thread_scan_library(library_dir: String):
	_scan_directory(library_dir, library_dir)
	found_models.sort_custom(_sort_models_by_name)


func _thread_scan_queued_models():
	while run_model_scan_thread:
		var next_model: Model = model_scan_queue.pop_front()
		if next_model == null:
			# Wait for a model to be available
			OS.delay_msec(1)
		else:
			next_model.scan_directory()
			found_models.append(next_model)
			amount_processed += 1


func _scan_directory(path: String, library_dir: String):
	var dir = DirAccess.open(path)
	if !dir:
		return
	
	# Directory exists. Scan it.
	var files = Array(dir.get_files())
	files = files.filter(_filter_ignored_files)
	
	if files.size() > 0:
		# This is a model directory
		var new_model = Model.new(path, library_dir)
		model_scan_queue.append(new_model)
		amount_found += 1
	else:
		# Might contain sub directories
		var subdirs = dir.get_directories()
		
		# First check if this is a "thingyverse" folder structure.
		for subdir in subdirs:
			if subdir.to_lower() == "files":
				# Thingyverse.
				var new_model = Model.new(path, library_dir)
				model_scan_queue.append(new_model)
				amount_found += 1
				return
		
		for subdir in subdirs:
			var new_path = "%s/%s" % [path, subdir]
			_scan_directory(new_path, library_dir)


## Returns true if the file should be included, false otherwise.
func _filter_ignored_files(file: String) -> bool:
	for extension in ignored_extensions:
		if file.to_lower().ends_with(extension):
			return false
	
	return true


func _sort_models_by_name(a: Model, b: Model) -> bool:
	return Utils.sort_string_natural_order(a.name, b.name)
