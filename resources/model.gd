class_name Model
extends Resource

## Emitted when the cover image changes.
signal cover_image_changed()

## A single model can contain multiple stl files.

const CONFIG_FILE_NAME := "config.moa"

@export var directory: String
## The directory relative to the library root
@export var relative_directory: String
@export var name: String

## Relative path of all printable files found in the directory.
@export var printable_files: Array[String]
## Relative path of all non-model files found in the directory.
## Excludes images that have the same file name as a 3d model,
## Those are in `rendered_files`
@export var misc_files: Array[String]
## When an image has the same directory/name as a renderable file, it is
## put in this list.
@export var rendered_files: Array[String]

## Relative path to the cover image.
@export var cover_image_path: String = ""
## Cover image, if available
@export var cover_image: ImageTexture = null

## --- User config that is saved in the directory on-change ---

## Which way "up" the model should be displayed by default.
var default_orientation: Utils.ModelOrientation = Utils.ModelOrientation.Z_UP

## If this is empty, or the file cannot be loaded, the cover image is autodetected.
var cover_image_override: String = ""

## --- /User config ---

var supported_image_extensions := [".jpg", ".jpeg", ".png", ".webp"]
## These extensions can be put on the 3d printer. Others can't.
var printable_file_extensions := [".stl", ".obj", ".3mf"]

## The p_directory and library directory should be absolute paths.
func _init(p_directory = "", library_directory = ""):
	directory = p_directory
	relative_directory = p_directory.trim_prefix(library_directory)
	name = directory.split("/")[-1].split(".")[0]

## Scans the model directory for relevant info.
## Call this once after creating the resource.
func scan_directory():
	printable_files = []
	misc_files = []
	rendered_files = []
	cover_image_path = ""
	cover_image = null
	
	_load_config_or_default()
	
	_scan_subdirectory(directory, "")
	
	printable_files.sort_custom(Utils.sort_string_natural_order)
	misc_files.sort_custom(Utils.sort_string_natural_order)
	rendered_files.sort_custom(Utils.sort_string_natural_order)
	
	find_and_load_cover_image()

func _load_config_or_default():
	var config = ConfigFile.new()
	# We do not care if the config loads correctly or not.
	# Because if there is no config, we want to fall back to defaults anyway.
	config.load("%s/%s" % [directory, CONFIG_FILE_NAME])
	
	default_orientation = config.get_value("main", "default_orientation", Utils.ModelOrientation.Z_UP)
	cover_image_override = config.get_value("main", "cover_image_override", "")

## Call this only when the config is changed.
func save_config():
	var config = ConfigFile.new()
	
	config.set_value("main", "default_orientation", default_orientation)
	config.set_value("main", "cover_image_override", cover_image_override)
	
	config.save("%s/%s" % [directory, CONFIG_FILE_NAME])

## Returns true if the model matches the search parameters.
func matches_search(search_text: String) -> bool:
	var dir_result := Utils.string_matches_search_pattern(relative_directory, search_text)
	if dir_result == Utils.StringMatchResult.SUCCESS:
		return true
	elif dir_result == Utils.StringMatchResult.FAIL_CONTAINS_NEGATIVE:
		# If the directory contains a negative, we don't need to look at the
		# individiual files. 
		return false
	
	# Directory did not match, look at the files instead.
	
	for file in printable_files:
		if Utils.string_matches_search_pattern(file, search_text) == Utils.StringMatchResult.SUCCESS:
			return true
	
	return false


func find_and_load_cover_image():
	# See if we can find a cover image.
	var potential_covers := []
	
	if !cover_image_override.is_empty():
		# The user has selected a cover and it exists, use that as first option.
		if FileAccess.file_exists("%s/%s" % [directory, cover_image_override]):
			potential_covers.append(cover_image_override)
		else:
			# Selected cover no longer exists.
			cover_image_override = ""
	
	potential_covers.append_array(misc_files)
	# If there is no cover in the standard images, we fall back to rendered ones.
	potential_covers.append_array(rendered_files)
	
	for file in potential_covers:
		for extension in supported_image_extensions:
			if file.ends_with(extension):
				if _load_cover_image(file) == OK:
					# Cover image loaded ok. Else we try the next one.
					cover_image_path = file
					cover_image_changed.emit()
					return
	
	# No cover image.
	cover_image_path = ""
	cover_image = null
	cover_image_changed.emit()

## Returns OK if the load was successful.
func _load_cover_image(relative_path: String) -> int:
	if relative_path.is_empty():
		return FAILED
	
	var absolute_image_path := "%s/%s" % [directory, relative_path]
	var cached_cover_path = "%s/%s.png" % [Utils.cover_image_cache_dir, Utils.hash_string(absolute_image_path)]
	
	# The modified time will be "0" if the image does not exist, so it works even if the cache does not exist.
	var image_is_newer_than_cache := FileAccess.get_modified_time(absolute_image_path) > FileAccess.get_modified_time(cached_cover_path)
	
	if !image_is_newer_than_cache && cover_image != null && cover_image_path == relative_path:
		# This cover image is already loaded and up to date.
		return OK
	
	var image = Image.new()
	
	var err := ERR_QUERY_FAILED
	if !image_is_newer_than_cache:
		# If the cache is up-to-date, try to load from cache.
		err = image.load(cached_cover_path)
	
	if err == OK:
		# Cached image loaded
		cover_image = ImageTexture.create_from_image(image)
	else:
		# No cached image, load the original.
		err = image.load(absolute_image_path)
	
		if err == OK:
			Utils.fit_image_proportional(image)
			
			# Cache the smaller version of the cover image.
			image.save_png(cached_cover_path)
			cover_image = ImageTexture.create_from_image(image)
	
	return err

func _scan_subdirectory(base_path: String, subdir: String):
	var dir = DirAccess.open("%s/%s" % [base_path, subdir])
	if !dir:
		return
	
	# Directory exists. Scan it.
	var new_files = Array(dir.get_files())
	while !new_files.is_empty():
		var next_file: String = new_files.pop_front()
		
		# This is protection against thingyverse wierdness, where sometimes
		# there are broken stl files in the images directory.
		if subdir.split("/")[-1] == "images" && next_file.ends_with(".stl"):
			# Do nothing
			continue
		
		if next_file == CONFIG_FILE_NAME:
			# Do not list the config in the list of files.
			continue
		
		var file_path := "%s/%s" % [subdir, next_file]
		
		if _is_file_printable(file_path):
			printable_files.append(file_path)
			
			# Check if there is a png with the same name.
			var rendered_png_file := Utils.strip_extension(next_file) + ".png"
			var position := new_files.find(rendered_png_file)
			
			var relative_path := "%s/%s" % [subdir, rendered_png_file]
			if position != -1:
				# There _is_ a rendered version of this renderable file.
				rendered_files.append(relative_path)
				# That file is now processed, skip it.
				new_files.remove_at(position)
			else:
				# Maybe we have already processed the file?
				position = misc_files.find(relative_path)
				if position != -1:
					misc_files.remove_at(position)
					rendered_files.append(relative_path)
		else:
			misc_files.append(file_path)
	
	# Scan subdirectories
	var subdirs = dir.get_directories()
	for new_dir in subdirs:
		var new_subdir = "%s/%s" % [subdir, new_dir]
		_scan_subdirectory(base_path, new_subdir)


func _is_file_printable(file: String) -> bool:
	for extension in printable_file_extensions:
		if file.to_lower().ends_with(extension):
			return true
	
	return false


## Call this if a new rendered file has been created in the model directory.
func new_rendered_file(relative_path: String):
	if rendered_files.find(relative_path) != -1:
		# Already have this.
		# But it might still be changed (for example due to orientation)
		# So recheck the cover image.
		find_and_load_cover_image()
		return
	
	rendered_files.append(relative_path)
	rendered_files.sort_custom(Utils.sort_string_natural_order)
	
	# The new file might be a cover image.
	find_and_load_cover_image()
