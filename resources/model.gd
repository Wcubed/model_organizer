class_name Model
extends Resource

## A single model can contain multiple stl files.

@export var directory: String
@export var name: String
## Relative path of all files found in the directory.
@export var files: Array[String]
## Relative path to the cover image.
@export var cover_image_path: String = ""
## Cover image, if available
@export var cover_image: ImageTexture = null

var supported_image_extensions := [".jpg", ".jpeg", ".png", ".webp"]

func _init(p_directory = ""):
	directory = p_directory
	name = directory.split("/")[-1].split(".")[0]

## Scans the model directory for relevant info.
## Call this once after creating the resource.
func scan_directory():
	files = []
	cover_image_path = ""
	cover_image = null
	
	_scan_subdirectory(directory, "")
	
	_find_and_load_cover_image()

## Returns true if the model matches the search parameters.
func matches_search(search_text: String) -> bool:
	var text_items = search_text.to_lower().split(" ")
	
	for item in text_items:
		if item.is_empty():
			continue
		
		if !_contains_text_item(item):
			# Item is not in the model.
			return false
	# All items are in the model, this model matches.
	return true

## Returns true if the model contains this text somewhere.
## Item should be lowercase.
func _contains_text_item(item: String) -> bool:
	if name.to_lower().contains(item) || directory.to_lower().contains(item):
		return true
	
	for file in files:
		if file.to_lower().contains(item):
			return true
	
	return false

func _find_and_load_cover_image():
	# See if we can find a cover image.
	for file in files:
		for extension in supported_image_extensions:
			if file.ends_with(extension):
				cover_image_path = file
				
				if _load_cover_image() == OK:
					# Cover image loaded ok. Else we try the next one.
					return
	
	# No cover image.
	cover_image_path = ""

## Returns OK if the load was successful.
func _load_cover_image() -> int:
	if cover_image_path.is_empty():
		return FAILED
	
	var absolute_image_path := "%s/%s" % [directory, cover_image_path]
	var cached_cover_path = "%s/%s.png" % [Utils.cover_image_cache_dir, Utils.hash_string(cover_image_path)]
	
	# First try to load the cached image.
	var image = Image.new()
	var err := image.load(cached_cover_path)
	
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
	for file in new_files:
		# This is protection against thingyverse wierdness, where sometimes
		# there are broken stl files in the images directory.
		if subdir.split("/")[-1] == "images" && file.ends_with(".stl"):
			continue
		
		files.append("%s/%s" % [subdir, file])
	
	# Scan subdirectories
	var subdirs = dir.get_directories()
	for new_dir in subdirs:
		var new_subdir = "%s/%s" % [subdir, new_dir]
		_scan_subdirectory(base_path, new_subdir)
